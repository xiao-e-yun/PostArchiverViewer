pub mod category;
pub mod post;
pub mod posts;
pub mod relation;
pub mod summary;
pub mod utils;

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
    routing::get,
};
use cached::{TimedCache, TimedSizedCache};
use category::Category;
use post_archiver::{Author, Collection, Platform, Tag, manager::PostArchiverManager};
use posts::SearchQuery;
use serde::Deserialize;
use summary::get_summary_api;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    manager: Arc<Mutex<PostArchiverManager>>,
    caches: Arc<Caches>,
}

#[derive(Debug)]
pub struct Caches {
    pub tables: Mutex<TimedCache<&'static str, usize>>,
    pub posts: Mutex<TimedSizedCache<SearchQuery, usize>>,
}

impl AppState {
    pub fn manager(&self) -> std::sync::MutexGuard<'_, PostArchiverManager> {
        self.manager.lock().unwrap()
    }
}

pub fn get_api_router(config: &Config) -> Router<()> {
    let path = config.path.clone();

    #[allow(unused_mut)]
    let mut manager = connect_database(path.as_path());

    let manager = Arc::new(Mutex::new(manager));
    let state = AppState {
        caches: Arc::new(Caches {
            tables: Mutex::new(TimedCache::with_lifespan(60 * 60 * 12)),
            posts: Mutex::new(TimedSizedCache::with_size_and_lifespan(256, 60 * 60 * 12)),
        }),
        manager,
    };

    let router = Router::new()
        .route("/summary", get(get_summary_api))
        .route("/redirect", get(get_redirect_api));

    let router = posts::wrap_posts_route(router);
    let router = Tag::wrap_category_route(router);
    let router = Author::wrap_category_route(router);
    let router = Platform::wrap_category_route(router);
    let router = Collection::wrap_category_route(router);

    router.fallback(StatusCode::NOT_FOUND).with_state(state)
}

pub fn connect_database(path: &Path) -> PostArchiverManager {
    PostArchiverManager::open(path).unwrap().unwrap()
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
        Some(id) => format!("/posts/{id}"),
        None => url,
    };

    Ok(Redirect::permanent(&url))
}
