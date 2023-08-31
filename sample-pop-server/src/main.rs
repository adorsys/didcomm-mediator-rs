use sample_pop_server::{app, didgen, util::KeyStore};

use axum::Server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable tracing
    config_tracing();

    // Generate keystore (and its DID document) if not available
    if KeyStore::latest().is_none() {
        didgen::didgen().expect("Failed to generate an initial keystore and its DID document.");
    };

    // Start server
    let port = std::env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    tracing::info!("listening on {addr}");
    Server::bind(&addr)
        .serve(app().into_make_service())
        .await
        .unwrap();
}

fn config_tracing() {
    use tracing::Level;
    use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

    let tracing_layer = tracing_subscriber::fmt::layer();
    let filter = filter::Targets::new()
        .with_target("tower_http::trace::on_response", Level::DEBUG)
        .with_target("tower_http::trace::on_request", Level::DEBUG)
        .with_target("tower_http::trace::make_span", Level::DEBUG)
        .with_default(Level::INFO);

    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .init();
}
