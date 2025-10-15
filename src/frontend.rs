use std::sync::Arc;

use axum::{extract::State, http::Uri, response::{Html, IntoResponse}, routing::get, Router};
use rust_embed::Embed;
use tracing::info;

use crate::config::PublicConfig;

#[derive(Embed)]
#[folder = "frontend/dist/"]
struct Assets;

pub fn frontend(config: &PublicConfig) -> Router<()> {
    if cfg!(debug_assertions) {
        use axum_reverse_proxy::ReverseProxy;
        let proxy: ReverseProxy = ReverseProxy::new("/", "http://localhost:5173");
        info!("Running in debug mode");
        info!("Proxying to localhost:5173");

        let config = config.clone();
        let get_config = async |State(config): State<PublicConfig>| axum::Json(config);

        Router::from(proxy)
            .route("/config.json", get(get_config))
            .with_state(config)
    } else {
        let index_html = load_index_html(config);
        Router::new()
            .fallback(get(static_handler))
            .with_state(index_html)
    }
}

async fn static_handler(State(index_html): State<Arc<String>>, uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if !(path.is_empty() || path == INDEX_HTML)
        && let Some(content) = Assets::get(path)
    {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let headers = [(axum::http::header::CONTENT_TYPE, mime.as_ref())];
        (headers, content.data).into_response()
    } else {
        let index_html = index_html.as_str().to_string();
        Html(index_html).into_response()
    }
}

const INDEX_HTML: &str = "index.html";
fn load_index_html(config: &PublicConfig) -> Arc<String> {
    let file = Assets::get(INDEX_HTML).unwrap();
    let text = String::from_utf8(file.data.to_vec()).unwrap();

    // Replace placeholder to real public config
    let config = serde_json::to_string(config).unwrap();
    let script = format!("<script>window.PUBLIC_CONFIG={config}</script>");
    Arc::new(text.replace("<!--PUBLIC_CONFIG-->", &script))
}
