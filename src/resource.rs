use axum::Router;
use tower_http::services::ServeDir;

pub fn get_resource_router<P: AsRef<std::path::Path>>(path: P) -> Router {
    let serve_dir = ServeDir::new(&path);
    Router::new().fallback_service(serve_dir)
}