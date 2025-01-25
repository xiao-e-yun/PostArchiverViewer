use axum::{http::Uri, response::IntoResponse, routing::get, Router};
use axum_reverse_proxy::ReverseProxy;
use rust_embed::Embed;
use tracing::info;

static INDEX_HTML: &str = "index.html";

#[derive(Embed)]
#[folder = "frontend/dist/"]
struct Assets;

pub fn frontend() -> Router {
    if cfg!(debug_assertions) {
        let proxy: ReverseProxy = ReverseProxy::new("/", "http://localhost:5173");
        info!("Running in debug mode");
        info!("Proxying to localhost:5173");
        proxy.into()
    } else {
        Router::new().fallback(get(static_handler))
    }
}

pub enum FrontendService {
    Proxy(ReverseProxy),
    Static,
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return index_html().await;
    }

    return match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => {
            if path.contains('.') {
                return axum::http::StatusCode::NOT_FOUND.into_response();
            }

            index_html().await
        }
    };

    async fn index_html() -> axum::response::Response {
        let file = Assets::get(INDEX_HTML).unwrap();
        axum::response::Html(file.data).into_response()
    }
}
