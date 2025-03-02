mod api;
pub mod config;
pub mod frontend;
pub mod images;
pub mod resource;

use api::get_api_router;
use clap::Parser;
use config::Config;
use dotenv::dotenv;
use frontend::frontend;
use images::get_images_router;
use resource::get_resource_router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    dotenv().ok();
    let config = Config::parse();

    if !config.path.join("post-archiver.db").exists() {
        error!("Post Archiver is not found");
        return;
    }

    let images_router = get_images_router(&config);
    let resource_router = get_resource_router(&config);
    let api_router = get_api_router(&config);

    let app = frontend()
        .nest("/api", api_router)
        .nest("/images", images_router)
        .nest("/resource", resource_router)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::new().allow_origin(Any))
                .layer(CompressionLayer::new()),
        );

    let port = config.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}
