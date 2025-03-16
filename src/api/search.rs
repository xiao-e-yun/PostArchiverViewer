use axum::{extract::State, http::StatusCode};
use axum_extra::extract::Query;
use post_archiver::Post;
use rusqlite::params;
use serde::Deserialize;

#[cfg(feature = "full-text-search")]
use crate::config::Config;
#[cfg(feature = "full-text-search")]
use tracing::info;

use super::{
    generate_pagination,
    utils::{FromRow, PostMiniJson},
    APIResponse, AppState, AuthorPostsJson,
};

#[cfg(feature = "full-text-search")]
pub fn sync_search_api(config: &Config, conn: &rusqlite::Connection) -> bool {
    let old_status = conn
        .query_row(
            "SELECT value FROM _post_archiver_viewer WHERE future = 'search-full-text'",
            [],
            |row| row.get::<_, bool>(0),
        )
        .unwrap();

    let status = config.futures.full_text_search.unwrap_or(old_status);
    let changed = old_status != status;

    info!(
        "search-full-text: {} {}",
        if status { "enabled" } else { "disabled" },
        if changed { "(changed)" } else { "" }
    );

    if changed {
        if status {
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
        } else {
            info!("delete search table");
            conn.execute_batch(
                    "
                    BEGIN;
                    INSERT OR REPLACE INTO _post_archiver_viewer (future, value) VALUES ('search-full-text', 0);
                    DROP TABLE _posts_fts;
                    COMMIT;
                ",
                )
                .unwrap();
        }

        info!("cleanup database");
        conn.execute_batch("VACUUM;").unwrap();
    }

    if status {
        info!("rebuilt full-text search");
        conn.execute_batch(
            "
                INSERT INTO _posts_fts(_posts_fts) VALUES('rebuild');
                ",
        )
        .unwrap();
    }

    status
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
    let full_text_search = state.full_text_search();

    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let pagination_sql = generate_pagination(query.limit, query.page);
    let search = generate_search(full_text_search, &query.search);
    let tags = generate_tags(&query.tags);
    let tags_sql = generate_search_tags_sql(tags);

    let sql = generate_search_sql(full_text_search, &search, &tags_sql, &pagination_sql);

    let params = if !search.is_empty() {
        params![search]
    } else {
        params![]
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
                    let sql = generate_search_total_sql(full_text_search, &search, &tags_sql);

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

fn generate_search_sql(
    full_text_search: bool,
    search: &str,
    tags: &str,
    pagination: &str,
) -> String {
    if search.is_empty() {
        format!(
            "
        SELECT p.*
        FROM posts p
        {}
        ORDER BY updated DESC {}
        ",
            tags, pagination
        )
    } else if full_text_search {
        format!(
            "
        SELECT p.*
        FROM posts p
        JOIN (
            SELECT rowid
            FROM _posts_fts
            WHERE _posts_fts MATCH ?
        ) fts ON p.id = fts.rowid
        {}
        ORDER BY updated DESC {}
        ",
            tags, pagination
        )
    } else {
        format!(
            "
        SELECT *
        FROM posts p
        {}
        WHERE title LIKE ?
        ORDER BY updated DESC {}
        ",
            tags, pagination
        )
    }
}

fn generate_search_total_sql(full_text_search: bool, search: &str, tags: &str) -> String {
    if search.is_empty() {
        format!(
            "
        SELECT count()
        FROM posts p
        {}
        ",
            tags
        )
    } else if full_text_search {
        format!(
            "
        SELECT count()
        FROM posts p
        JOIN (
            SELECT rowid
            FROM _posts_fts
            WHERE _posts_fts MATCH ?
        ) fts ON p.id = fts.rowid
        {}
        ",
            tags
        )
    } else {
        format!(
            "
        SELECT count()
        FROM posts p
        {}
        WHERE title LIKE ?
        ",
            tags
        )
    }
}

fn generate_search_tags_sql(tags: Vec<String>) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(
            "
        JOIN (
            SELECT post
            FROM post_tags
            WHERE tag IN ({})
            GROUP BY post
            HAVING COUNT(DISTINCT tag) = {}
        ) pt ON p.id = pt.post
        ",
            tags.join(","),
            tags.len(),
        )
    }
}

fn generate_search(full_text_search: bool, search: &Option<String>) -> String {
    let Some(search) = &search else {
        return String::new();
    };

    if full_text_search {
        format!("\"{}\"", search)
    } else {
        format!("%{}%", search)
    }
}

fn generate_tags(tags: &[u32]) -> Vec<String> {
    if tags.is_empty() {
        vec![]
    } else {
        tags.iter().map(|x| x.to_string()).collect::<Vec<String>>()
    }
}
