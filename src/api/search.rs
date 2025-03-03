use std::mem;

use axum::{extract::State, http::StatusCode};
use axum_extra::extract::Query;
use post_archiver::Post;
use rusqlite::{params, Connection};
use serde::Deserialize;
use tracing::info;

use crate::config::{Config, Status};

use super::{
    utils::{FromRow, PostMiniJson},
    APIResponse, AppState, AuthorPostsJson,
};

pub fn sync_search_api(config: &Config, conn: &mut Connection) -> bool {
    let future = config.futures.search_full_text;

    let old_status = conn
        .query_row(
            "SELECT value FROM _post_archiver_viewer WHERE future = 'search-full-text'",
            [],
            |row| row.get::<_, u8>(0).map(Status::from_u8),
        )
        .unwrap();

    let status = future.or(Some(old_status)).or(Some(Status::Off)).unwrap();
    let changed = old_status != status;

    info!(
        "search-full-text: {} {}",
        status.enabled(),
        if changed { "(changed)" } else { "" }
    );

    if status.is_on() {
        info!("initializing full-text search");
        let dir = tempfile::tempdir().unwrap();
        libsimple::enable_auto_extension().unwrap();
        libsimple::release_dict(&dir).unwrap();

        let old = mem::replace(conn, Connection::open(conn.path().unwrap()).unwrap());
        old.close().unwrap();
        libsimple::set_dict(&conn, &dir).unwrap();

        if !changed {
            info!("rebuilt full-text search");
            conn.execute("INSERT INTO _posts_fts(_posts_fts) VALUES('rebuild')", [])
                .unwrap();
        }
    }

    if !changed {
        return status.is_on();
    };

    match status {
        Status::On => {
            info!("creating search table");
            conn.execute_batch(
                "
                BEGIN;
                INSERT OR REPLACE INTO _post_archiver_viewer (future, value) VALUES ('search-full-text', 1);
                CREATE VIRTUAL TABLE _posts_fts USING fts5(title, content, content=posts, content_rowid=id, tokenize = 'simple');
                COMMIT;
            "
            )
            .unwrap();

            info!("rebuilt full-text search");
            conn.execute_batch(
                "
            INSERT INTO _posts_fts(_posts_fts) VALUES('rebuild');
            VACUUM;
            ",
            )
            .unwrap();
        }
        Status::Off => {
            conn.execute_batch(
                "
                BEGIN;
                INSERT OR REPLACE INTO _post_archiver_viewer (future, value) VALUES ('search-full-text', 0);
                DROP TABLE _posts_fts;
                COMMIT;
            ",
            )
            .unwrap();

            info!("rebuilt database");
            conn.execute_batch("VACUUM;").unwrap();
        }
    };

    status.is_on()
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    search: Option<String>,
    #[serde(default)]
    tags: Vec<u32>,
    limit: Option<u32>,
    page: Option<u32>,
}

pub async fn get_search_api(
    Query(query): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Result<APIResponse<AuthorPostsJson>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let pagination_sql = match (query.limit, query.page) {
        (Some(limit), Some(page)) => format!("LIMIT {} OFFSET {}", limit, page * limit),
        _ => "".to_string(),
    };

    let tags = format!(
        "({})",
        query
            .tags
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );

    let tags_count = query.tags.len() as u32;
    let search = if state.full_text_search {
        if let Some(search) = &query.search {
            format!("\"{}\"", search)
        } else {
            "\"\"".to_string()
        }
    } else {
        if let Some(search) = &query.search {
            format!("%{}%", search)
        } else {
            "%".to_string()
        }
    };

    let (join_sql, where_sql) = if query.search.is_none() {
        ("", "")
    } else if state.full_text_search {
        (
            "JOIN (
                SELECT rowid
                FROM _posts_fts
                WHERE _posts_fts MATCH ?
            ) fts ON p.id = fts.rowid",
            "",
        )
    } else {
        ("", "WHERE p.title LIKE ?")
    };

    let params = if query.search.is_some() {
        params![search]
    } else {
        params![]
    };

    let sql = if tags_count > 0 {
        format!(
            "
                SELECT p.*
                FROM posts p
                {}
                JOIN (
                    SELECT post
                    FROM post_tags
                    WHERE tag IN {}
                    GROUP BY post
                    HAVING COUNT(DISTINCT tag) = {}
                ) pt ON p.id = pt.post 
                {}
                ORDER BY updated DESC {}
                ",
            join_sql, tags, tags_count, where_sql, pagination_sql
        )
    } else {
        format!(
            "
                SELECT *
                FROM posts p
                {}
                {}
                ORDER BY updated DESC {}
                ",
            join_sql, where_sql, pagination_sql
        )
    };

    let mut stmt = tx
        .prepare(&sql)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query(params)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut posts = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let post = Post::from_row(&state, row).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let post = PostMiniJson::resolve(&state, &tx, post)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match post {
            Some(post) => posts.push(post),
            None => continue,
        }
    }

    let total = match pagination_sql.as_str() {
        "" => posts.len() as u32,
        _ => {
            let mut cache = state.cache.lock().unwrap();
            let key = postcard::to_allocvec(&(&query.search, query.tags)).unwrap();
            match cache.get(&key) {
                Some(total) => *total,
                None => {
                    let sql = if tags_count > 0 {
                        format!(
                            "
                                SELECT count()
                                FROM posts p
                                {}
                                JOIN (
                                    SELECT post
                                    FROM post_tags
                                    WHERE tag IN {}
                                    GROUP BY post
                                    HAVING COUNT(DISTINCT tag) = {}
                                ) pt ON p.id = pt.post 
                                {}
                                ",
                            join_sql, tags, tags_count, where_sql
                        )
                    } else {
                        if query.search.is_none() {
                            format!("SELECT count() FROM posts")
                        } else if state.full_text_search {
                            format!("SELECT count() FROM _posts_fts WHERE _posts_fts MATCH ?")
                        } else {
                            format!("SELECT count() FROM posts WHERE title LIKE ?")
                        }
                    };

                    let total: u32 = tx
                        .query_row(&sql, params, |row| row.get(0))
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    cache.put(key, total);
                    total
                }
            }
        }
    };

    let data = AuthorPostsJson { posts, total };

    Ok(APIResponse { data })
}
