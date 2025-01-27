pub mod utils;

use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use post_archiver::{Author, AuthorId, Post, PostId};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ts_rs::TS;
use utils::{AuthorJson, FromRow, PostJson, PostMiniJson, TagJson};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    conn: Arc<Mutex<Connection>>,
    config: Config,
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
    let conn = Connection::open(path.join("post-archiver.db")).unwrap();
    let conn = Arc::new(Mutex::new(conn));
    let state = AppState {
        config: config.clone(),
        conn,
    };

    Router::new()
        .route("/authors", get(get_authors_api))
        .route("/author", get(get_author_api))
        .route("/post", get(get_post_api))
        .route("/tags", get(get_tags_api))
        .route("/info", get(get_info_api))
        .fallback(StatusCode::NOT_FOUND)
        .with_state(state)
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
pub struct PostsQuery {
    author: u32,
}

async fn get_author_api(
    Query(query): Query<PostsQuery>,
    State(state): State<AppState>,
) -> Result<APIResponse<AuthorFullJson>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut stmt = tx
        .prepare_cached("SELECT * FROM posts WHERE author = ? ORDER BY updated DESC")
        .unwrap();
    let mut rows = stmt.query([query.author]).unwrap();

    let mut posts = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let post = Post::from_row(&state, row).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let post = PostMiniJson::resolve(&state, &tx, post)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        posts.push(post);
    }

    let author = AuthorJson::from_id(&state, &tx, AuthorId(query.author))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(APIResponse {
        data: AuthorFullJson { author, posts },
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthorFullJson {
    #[serde(flatten)]
    author: AuthorJson,
    posts: Vec<PostMiniJson>,
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
    Ok(APIResponse { data })
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

async fn get_info_api(State(state): State<AppState>) -> Result<APIResponse<Value>, StatusCode> {
    let mut conn = state.conn();
    let tx = conn
        .transaction()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let count_authors: u32 = tx
        .query_row("SELECT COUNT() FROM authors", [], |row| row.get(0))
        .unwrap();
    let count_posts: u32 = tx
        .query_row("SELECT COUNT() FROM posts", [], |row| row.get(0))
        .unwrap();
    let count_files: u32 = tx
        .query_row("SELECT COUNT() FROM file_metas", [], |row| row.get(0))
        .unwrap();

    Ok(APIResponse {
        data: json!({
            "authors":count_authors,
            "posts":count_posts,
            "files":count_files,
        }),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InfoJson {
    authors: u32,
    posts: u32,
    files: u32,
}
