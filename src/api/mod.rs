pub mod category;
pub mod error;
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
use error::{ApiError, ApiResult};
use post_archiver::{
    manager::PostArchiverManager,
    query::{Pagination, PostArchiverQueryExt, Query as _},
};
use serde::Deserialize;
use summary::get_summary_api;
use url::Url;
use utils::post_preview::PostPreview;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    manager: Arc<Mutex<PostArchiverManager>>,
}

impl AppState {
    pub fn manager(&self) -> std::sync::MutexGuard<'_, PostArchiverManager> {
        self.manager.lock().unwrap()
    }
}

pub fn get_api_router(config: &Config) -> Router<()> {
    let path = config.path.clone();

    let manager = connect_database(path.as_path());
    let state = AppState {
        manager: Arc::new(Mutex::new(manager)),
    };

    let router = Router::new()
        .route("/summary", get(get_summary_api))
        .route("/redirect", get(get_redirect_api));

    let router = posts::wrap_posts_route(router);
    let router = category::wrap_category_routes(router);

    router.fallback(StatusCode::NOT_FOUND).with_state(state)
}

pub fn connect_database(path: &Path) -> PostArchiverManager {
    let manager = PostArchiverManager::open(path).unwrap().unwrap();

    // Tune SQLite for a read-mostly viewer workload. All of these are
    // per-connection pragmas that do NOT modify the database file, so they are
    // safe even though the archive is owned/written by the archiver process.
    manager
        .conn
        .execute_batch(
            "
            PRAGMA query_only = ON;       -- read-only guard: reject any accidental writes
            PRAGMA busy_timeout = 5000;   -- wait up to 5s on a lock instead of erroring
            PRAGMA cache_size = -65536;   -- ~64MB page cache (negative value = KiB)
            PRAGMA mmap_size = 268435456; -- 256MB memory-mapped reads, fewer syscalls
            PRAGMA temp_store = MEMORY;   -- keep sorts/temp tables in memory
            ",
        )
        .expect("failed to apply SQLite pragmas");

    manager
}

#[derive(Debug, Deserialize)]
pub struct RedirectQuery {
    url: String,
}

async fn get_redirect_api(
    Query(query): Query<RedirectQuery>,
    State(state): State<AppState>,
) -> ApiResult<Redirect> {
    let url = query.url;

    let parsed = Url::parse(&url);
    let scheme = parsed.as_ref().map(Url::scheme);
    if !matches!(scheme, Ok("http") | Ok("https")) {
        return Err(ApiError::BadRequest);
    }

    let manager = state.manager();
    let mut query = manager.posts().query();
    query.source.equals(&url);
    query.pagination = Pagination::default().limit(1);
    let post = query.query::<PostPreview>()?.into_iter().next();

    let url = match post {
        Some(post) => format!("/posts/{}", post.id),
        None => url,
    };

    Ok(Redirect::permanent(&url))
}
