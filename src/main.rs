mod api;
pub mod config;
pub mod frontend;
pub mod images;
pub mod resource;

use api::get_api_router;
use clap::Parser;
use config::Config;
use console::style;
use dotenv::dotenv;
use frontend::frontend;
use images::get_images_router;
use local_ip_address::local_ip;
use qrcode::{render::unicode, QrCode};
use resource::get_resource_router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};
use tracing_subscriber::fmt::{self, time::UtcTime};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let timer = UtcTime::new(
        time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap(),
    );

    let format = fmt::format()
        .with_level(true)
        .with_target(false)
        .with_timer(timer);

    tracing_subscriber::fmt().event_format(format).init();

    dotenv().ok();
    let config = Config::parse();

    info!("# {} #", style("Post Archiver").green().bold());
    info!("==========================");
    info!("Version {}", style(format!("v{}", VERSION)).green().bold());
    info!(
        "PostArchiver {}",
        style(format!("v{}", post_archiver::utils::VERSION))
            .green()
            .bold()
    );

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
    info!("");

    info!(
        " {} http://localhost:{} ",
        style("Local").green().bold(),
        port
    );
    if let Ok(addr) = local_ip() {
        let url = format!("http://{}:{}", addr, port);
        info!(" {} {}", style("Network").green().bold(), url);

        let qrcode = QrCode::new(url.clone()).unwrap();
        let qrcode = qrcode
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Dark)
            .light_color(unicode::Dense1x2::Light)
            .quiet_zone(false)
            .build();

        info!("");
        for line in qrcode.lines() {
            info!(" {}", line);
        }
        info!("");
    }

    info!(
        "Press {} to stop the server",
        style("Ctrl + C").green().bold()
    );
    axum::serve(listener, app).await.unwrap();
}
