pub mod search;
pub mod utils;

use std::{
    num::NonZero,
    path::Path,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use lru::LruCache;
use post_archiver::{Author, AuthorId, Post, PostId};
use rusqlite::Connection;
use search::get_search_api;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ts_rs::TS;
use utils::{AuthorJson, FromRow, PostJson, PostMiniJson, TagJson};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    conn: Arc<Mutex<Connection>>,
    cache: Arc<Mutex<LruCache<Vec<u8>, u32>>>,
    config: Config,
    #[cfg(feature = "full-text-search")]
    full_text_search: bool,
}

impl AppState {
    fn full_text_search(&self) -> bool {
        #[cfg(feature = "full-text-search")]
        let value = self.full_text_search;
        #[cfg(not(feature = "full-text-search"))]
        let value = false;
        value
    }
}

impl AppState {
    pub fn conn(&self) -> std::sync::MutexGuard<Connection> {
        self.conn.lock().unwrap()
    }
    pub fn static_url(&self, mime: &str) -> String {
        let resource_url = self
            .config
            .resource_url
            .clone()
            .unwrap_or("/resource".to_string());
        let images_url = self
            .config
            .images_url
            .clone()
            .unwrap_or("/images".to_string());

        let url = match mime.starts_with("image/") {
            true => images_url,
            false => resource_url,
        };

        match url.strip_suffix('/') {
            Some(url) => url.to_string(),
            None => url,
        }
    }
}

#[derive(Serialize)]
pub struct APIResponse<T> {
    data: T,
}

impl<T: Serialize> IntoResponse for APIResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self.data).into_response()
    }
}

pub fn get_api_router(config: &Config) -> Router {
    let path = config.path.clone();

    let conn = connect_database(path.as_path());

    // Create futures table
    conn.execute_batch("
    CREATE TABLE IF NOT EXISTS _post_archiver_viewer (future TEXT PRIMARY KEY, value INTEGER DEFAULT 0, extra TEXT DEFAULT '{}');
    INSERT OR IGNORE INTO _post_archiver_viewer (future) VALUES ('search-full-text');
    ").unwrap();

    #[cfg(feature = "full-text-search")]
    let full_text_search = search::sync_search_api(&config, &conn);

    let conn = Arc::new(Mutex::new(conn));
    let state = AppState {
        cache: Arc::new(Mutex::new(LruCache::new(NonZero::new(20).unwrap()))),
        config: config.clone(),
        conn,
        #[cfg(feature = "full-text-search")]
        full_text_search,
    };

    Router::new()
        .route("/authors", get(get_authors_api))
        .route("/author", get(get_author_api))
        .route("/search", get(get_search_api))
        .route("/posts", get(get_posts_api))
        .route("/post", get(get_post_api))
        .route("/tags", get(get_tags_api))
        .route("/info", get(get_info_api))
        .fallback(StatusCode::NOT_FOUND)
        .with_state(state)
}

pub fn connect_database(path: &Path) -> Connection {
    #[cfg(feature = "full-text-search")]
    let dir = {
        let dir = tempfile::tempdir().unwrap();
        libsimple::enable_auto_extension().unwrap();
        libsimple::release_dict(&dir).unwrap();
        dir
    };

    let conn = Connection::open(path.join("post-archiver.db")).unwrap();

    #[cfg(feature = "full-text-search")]
    libsimple::set_dict(&conn, &dir).unwrap();

    conn
}

async fn get_authors_api(
    State(state): State<AppState>,
) -> Result<APIResponse<Vec<AuthorJson>>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut stmt = tx
        .prepare_cached("SELECT * FROM authors ORDER BY updated DESC")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query([])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut data = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let author =
            Author::from_row(&state, row).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let author = AuthorJson::resolve(&state, &tx, author)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        data.push(author);
    }

    Ok(APIResponse { data })
}

#[derive(Debug, Deserialize)]
pub struct AuthorQuery {
    author: u32,
}

async fn get_author_api(
    Query(query): Query<AuthorQuery>,
    State(state): State<AppState>,
) -> Result<APIResponse<AuthorJson>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = AuthorJson::from_id(&state, &tx, AuthorId(query.author))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match data {
        Some(data) => Ok(APIResponse { data }),
        None => return Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct PostsQuery {
    author: u32,
    limit: Option<u32>,
    page: Option<u32>,
}

async fn get_posts_api(
    Query(query): Query<PostsQuery>,
    State(state): State<AppState>,
) -> Result<APIResponse<AuthorPostsJson>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let pagination_sql = generate_pagination(query.limit, query.page);

    let sql = format!(
        "SELECT * FROM posts WHERE author = ? ORDER BY updated DESC {}",
        pagination_sql
    );
    let mut stmt = tx
        .prepare_cached(&sql)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query([query.author])
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
            let key = postcard::to_allocvec(&query.author).unwrap();
            match cache.get(&key) {
                Some(total) => *total,
                None => {
                    let sql = format!("SELECT count() FROM posts WHERE author = ?");
                    let total: u32 = tx
                        .query_row(&sql, [query.author], |row| row.get(0))
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthorPostsJson {
    posts: Vec<PostMiniJson>,
    total: u32,
}

#[derive(Debug, Deserialize)]
pub struct PostQuery {
    post: u32,
}

async fn get_post_api(
    Query(query): Query<PostQuery>,
    State(state): State<AppState>,
) -> Result<APIResponse<PostJson>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let id = PostId(query.post);
    let data = PostJson::from_id(&state, &tx, id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match data {
        Some(data) => Ok(APIResponse { data }),
        None => return Err(StatusCode::NOT_FOUND),
    }
}

async fn get_tags_api(
    State(state): State<AppState>,
) -> Result<APIResponse<Vec<TagJson>>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data = TagJson::all(&state, &tx).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(APIResponse { data })
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InfoJson {
    authors: u32,
    posts: u32,
    files: u32,
}

async fn get_info_api(State(state): State<AppState>) -> Result<APIResponse<Value>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let count_authors: u32 = tx
        .query_row("SELECT COUNT() FROM authors", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let count_posts: u32 = tx
        .query_row("SELECT COUNT() FROM posts", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let count_files: u32 = tx
        .query_row("SELECT COUNT() FROM file_metas", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(APIResponse {
        data: json!({
            "authors":count_authors,
            "posts":count_posts,
            "files":count_files,
        }),
    })
}

pub fn generate_pagination(limit: Option<u32>, page: Option<u32>) -> String {
    match (limit, page) {
        (Some(limit), Some(page)) => format!("LIMIT {} OFFSET {}", limit, page * limit),
        _ => "".to_string(),
    }
}
