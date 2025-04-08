pub mod search;
pub mod utils;

use std::{
    collections::HashMap,
    num::NonZero,
    path::Path,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
    routing::get,
    Json, Router,
};
use lru::LruCache;
use post_archiver::{
    manager::PostArchiverManager,
    utils::{author::GetAuthor, post::GetPost},
    Author, AuthorId, Post, PostId, PostTagId,
};
use search::get_search_api;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ts_rs::TS;
use utils::{list_tags, AuthorJson, FromRow, PostJson, PostMiniJson};

use crate::config::{Config, PublicConfig};

#[derive(Clone)]
pub struct AppState {
    manager: Arc<Mutex<PostArchiverManager>>,
    cache: Arc<Mutex<LruCache<Vec<u8>, u32>>>,
    config: Config,
    #[cfg(feature = "full-text-search")]
    full_text_search: bool,
}

impl AppState {
    pub fn manager(&self) -> std::sync::MutexGuard<PostArchiverManager> {
        self.manager.lock().unwrap()
    }
    fn full_text_search(&self) -> bool {
        #[cfg(feature = "full-text-search")]
        let value = self.full_text_search;
        #[cfg(not(feature = "full-text-search"))]
        let value = false;
        value
    }
}

pub fn get_api_router(config: &Config) -> Router {
    let path = config.path.clone();

    #[allow(unused_mut)]
    let mut manager = connect_database(path.as_path());

    #[cfg(feature = "full-text-search")]
    let full_text_search = search::sync_search_api(&config, &mut manager);

    let manager = Arc::new(Mutex::new(manager));
    let state = AppState {
        cache: Arc::new(Mutex::new(LruCache::new(NonZero::new(20).unwrap()))),
        config: config.clone(),
        manager,
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
        .route("/redirect", get(get_redirect_api))
        .route("/config.json", get(get_config_api))
        .fallback(StatusCode::NOT_FOUND)
        .with_state(state)
}

pub fn connect_database(path: &Path) -> PostArchiverManager {
    #[cfg(feature = "full-text-search")]
    let dir = {
        let dir = tempfile::tempdir().unwrap();
        libsimple::enable_auto_extension().unwrap();
        libsimple::release_dict(&dir).unwrap();
        dir
    };

    let manager = PostArchiverManager::open(path).unwrap().unwrap();

    #[cfg(feature = "full-text-search")]
    libsimple::set_dict(&manager.conn(), &dir).unwrap();

    manager
}

async fn get_authors_api(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuthorJson>>, StatusCode> {
    let manager = state.manager();
    let conn = manager.conn();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM authors ORDER BY updated DESC")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query([])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut data = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let author = Author::from_row(row).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let author =
            AuthorJson::resolve(&manager, author).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        data.push(author);
    }

    Ok(Json(data))
}

#[derive(Debug, Deserialize)]
pub struct AuthorQuery {
    author: u32,
}

async fn get_author_api(
    Query(query): Query<AuthorQuery>,
    State(state): State<AppState>,
) -> Result<Json<AuthorJson>, StatusCode> {
    let manager = state.manager();

    let author = AuthorId(query.author)
        .author(&manager)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    let data =
        AuthorJson::resolve(&manager, author).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(data))
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
) -> Result<Json<AuthorPostsJson>, StatusCode> {
    let manager = state.manager();
    let conn = manager.conn();

    let pagination_sql = generate_pagination(query.limit, query.page);

    let sql = format!(
        "SELECT * FROM posts WHERE author = ? ORDER BY updated DESC {}",
        pagination_sql
    );
    let mut stmt = conn
        .prepare_cached(&sql)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query([query.author])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut posts = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let post = Post::from_row(row).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let post =
            PostMiniJson::resolve(&manager, post).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
                    let sql = "SELECT count() FROM posts WHERE author = ?".to_string();
                    let total: u32 = conn
                        .query_row(&sql, [query.author], |row| row.get(0))
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    cache.put(key, total);
                    total
                }
            }
        }
    };

    let data = AuthorPostsJson { posts, total };

    Ok(Json(data))
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
) -> Result<Json<PostJson>, StatusCode> {
    let manager = state.manager();

    let id = PostId(query.post);
    let data = PostJson::resolve(
        &manager,
        id.post(&manager)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match data {
        Some(data) => Ok(Json(data)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_tags_api(
    State(state): State<AppState>,
) -> Result<Json<HashMap<PostTagId, String>>, StatusCode> {
    let manager = state.manager();

    let data = list_tags(&manager).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data = HashMap::from_iter(data.into_iter().map(|t| (t.id, t.name)));
    Ok(Json(data))
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InfoJson {
    authors: u32,
    posts: u32,
    files: u32,
}

async fn get_info_api(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let manager = state.manager();
    let conn = manager.conn();

    let count_authors: u32 = conn
        .query_row("SELECT COUNT() FROM authors", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let count_posts: u32 = conn
        .query_row("SELECT COUNT() FROM posts", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let count_files: u32 = conn
        .query_row("SELECT COUNT() FROM file_metas", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "authors":count_authors,
        "posts":count_posts,
        "files":count_files,
    })))
}

#[derive(Debug, Deserialize)]
pub struct RedirectQuery {
    url: String,
}

async fn get_redirect_api(
    Query(query): Query<RedirectQuery>,
    State(state): State<AppState>,
) -> Result<Redirect, StatusCode> {
    let url = query.url;

    let manager = state.manager();
    let conn = manager.conn();

    let mut stmt = conn
        .prepare_cached("SELECT id FROM posts WHERE source = ?")
        .unwrap();
    let id: Option<u32> = stmt.query_row([&url], |row| row.get(0)).ok();

    let url = match id {
        Some(id) => format!("/post/{}", id),
        None => url,
    };

    Ok(Redirect::permanent(&url))
}

pub async fn get_config_api(State(state): State<AppState>) -> Json<PublicConfig> {
    Json(state.config.public.clone())
}

pub fn generate_pagination(limit: Option<u32>, page: Option<u32>) -> String {
    match (limit, page) {
        (Some(limit), Some(page)) => format!("LIMIT {} OFFSET {}", limit, page * limit),
        _ => "".to_string(),
    }
}
