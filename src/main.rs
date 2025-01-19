mod api;

use api::get_api_router;
use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::Router,
};
use axum_reverse_proxy::ReverseProxy;
use rust_embed::Embed;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::services::ServeDir;
use tracing::info;

static INDEX_HTML: &str = "index.html";

#[derive(Embed)]
#[folder = "frontend/dist/"]
struct Assets;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let archiver_path = PathBuf::from("archiver");
    let static_server_url: Option<String> = None;

    let api = get_api_router(&archiver_path,static_server_url.clone());

    let static_server = {
        let router = Router::new();
        // if static_server_url is None, serve the files from the local directory
        if static_server_url.is_none() {
            info!("Serving static files from /archive/*");
            let service = ServeDir::new(archiver_path);
            router.fallback_service(service)
        } else {
            router.fallback(StatusCode::FORBIDDEN)
        }
    };

    let mut app = Router::new()
        .nest("/api", api)
        .nest("/archive", static_server);
    app = if cfg!(debug_assertions) {
        let proxy: ReverseProxy = ReverseProxy::new("/", "http://localhost:5173");
        info!("Running in debug mode");
        info!("Proxying to localhost:5173");
        app.merge(proxy)
    } else {
        app.fallback(static_handler)
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return index_html().await;
    }

    return match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return (StatusCode::NOT_FOUND, "404").into_response();
            }

            index_html().await
        }
    };

    async fn index_html() -> Response {
        let file = Assets::get(INDEX_HTML).unwrap();
        Html(file.data).into_response()
    }
}
