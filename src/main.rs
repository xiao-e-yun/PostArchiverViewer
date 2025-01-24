mod api;
pub mod frontend;
pub mod resource;
pub mod images;

use api::get_api_router;
use frontend::frontend;
use images::get_images_router;
use resource::get_resource_router;
use std::{net::SocketAddr, path::PathBuf};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let archiver_path = PathBuf::from("archiver");
    let static_server_url: Option<String> = None;

    let images_router = get_images_router(&archiver_path);
    let resource_router = get_resource_router(&archiver_path);
    let api_router = get_api_router(&archiver_path, static_server_url.clone());

    let app = frontend()
        .nest("/api", api_router)
        .nest("/images", images_router)
        .nest("/resource", resource_router)

        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::new().allow_origin(Any))
                .layer(CompressionLayer::new())
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
