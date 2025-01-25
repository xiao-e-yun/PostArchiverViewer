use axum::{http::StatusCode, Router};
use tower_http::services::ServeDir;
use tracing::info;

use crate::config::Config;

pub fn get_resource_router(config: &Config) -> Router {
    let router = Router::new();
    if config.resource_url.is_some() {
        info!("Resource URL is set, disabling resource router");
        return router.fallback(|| async { StatusCode::FORBIDDEN });
    }

    let serve_dir = ServeDir::new(&config.path);
    Router::new().fallback_service(serve_dir)
}
