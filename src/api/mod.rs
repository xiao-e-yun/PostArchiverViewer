pub mod category;
pub mod post;
pub mod relation;
pub mod search;
pub mod summary;
pub mod utils;

use std::{
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
use category::CategoryPostsApiRouter;
use mini_moka::sync::Cache;
use post_archiver::{
    manager::PostArchiverManager, Author, AuthorId, Collection, CollectionId, Platform, PlatformId,
    Tag, TagId,
};
use search::{get_search_api, SearchQuery};
use serde::Deserialize;
use summary::get_summary_api;

use crate::config::{Config, PublicConfig};

#[derive(Clone)]
pub struct AppState {
    manager: Arc<Mutex<PostArchiverManager>>,
    config: Config,
    caches: Caches,
    #[cfg(feature = "full-text-search")]
    full_text_search: bool,
}

#[derive(Clone, Debug)]
pub struct Caches {
    pub tables: Cache<&'static str, usize>,
    pub tags: Cache<TagId, usize>,
    pub authors: Cache<AuthorId, usize>,
    pub platforms: Cache<PlatformId, usize>,
    pub collections: Cache<CollectionId, usize>,
    pub search: Cache<SearchQuery, usize>,
}

impl AppState {
    pub fn manager(&self) -> std::sync::MutexGuard<'_,PostArchiverManager> {
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

pub fn get_api_router(config: &Config) -> Router<()> {
    let path = config.path.clone();

    #[allow(unused_mut)]
    let mut manager = connect_database(path.as_path());

    #[cfg(feature = "full-text-search")]
    let full_text_search = search::sync_search_api(config, &mut manager);

    let manager = Arc::new(Mutex::new(manager));
    let state = AppState {
        caches: Caches {
            tables: Cache::new(5),
            platforms: Cache::new(4),
            tags: Cache::new(8),
            collections: Cache::new(8),
            authors: Cache::new(16),
            search: Cache::new(32),
        },
        config: config.clone(),
        manager,
        #[cfg(feature = "full-text-search")]
        full_text_search,
    };

    let router = Router::new()
        .route("/search", get(get_search_api))
        .route("/summary", get(get_summary_api))
        .route("/redirect", get(get_redirect_api))
        .route("/config.json", get(get_config_api));

    let router = post::wrap_posts_route(router);
    let router = Tag::wrap_category_and_posts_route(router);
    let router = Author::wrap_category_and_posts_route(router);
    let router = Platform::wrap_category_and_posts_route(router);
    let router = Collection::wrap_category_and_posts_route(router);

    router.fallback(StatusCode::NOT_FOUND).with_state(state)
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
    libsimple::set_dict(manager.conn(), &dir).unwrap();

    manager
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

pub async fn get_config_api(State(state): State<AppState>) -> Json<PublicConfig> {
    Json(state.config.public.clone())
}
