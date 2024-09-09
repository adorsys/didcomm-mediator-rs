use axum::Server;
use didcomm_mediator::app;
use std::net::SocketAddr;
use secure_key_storage::SecretBox;
use zeroize::Zeroize;
use std::env;
use tracing::info;
use dotenv_flow::dotenv_flow;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load dotenv-flow variables
    dotenv_flow().ok();

    // Enable logging
    config_tracing();

    // Example: securely loading a secret key
    let api_key = load_secret_key().await;
    info!("Loaded API key.");

    // Start server
    let port = env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    info!("listening on {addr}");
    generic_server_with_graceful_shutdown(addr, api_key).await;
}

async fn load_secret_key() -> SecretBox<String> {
    // Simulate loading a secret key. Replace this with actual key loading.
    let secret_key = "supersecretapikey".to_string(); // This should be replaced with actual key retrieval logic
    SecretBox::new(secret_key)
}

async fn generic_server_with_graceful_shutdown(addr: SocketAddr, api_key: SecretBox<String>) {
    // Load plugins
    let (mut plugin_container, router) = app();

    // Use `api_key` securely as needed in your application, for example in your app's router setup.

    // Spawn task for server
    tokio::spawn(async move {
        Server::bind(&addr)
            .serve(router.into_make_service())
            .await
            .unwrap();
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("shutting down gracefully");
            let _ = plugin_container.unload();
        }
    };
}

fn config_tracing() {
    let tracing_layer = tracing_subscriber::fmt::layer();
    let filter = filter::Targets::new()
        .with_target("hyper::proto", tracing::Level::INFO)
        .with_target("tower_http::trace", tracing::Level::DEBUG)
        .with_default(tracing::Level::DEBUG);

    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .init();
}
