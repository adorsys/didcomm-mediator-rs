use axum::{routing::get, Router, Server};
use didcomm_mediator::app;
use eyre::{Result, WrapErr};
use prometheus::{Encoder, TextEncoder, Registry};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<()> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow()?;

    // Enable logging
    config_tracing();

    // Create a shared registry for Prometheus metrics
    let registry = Arc::new(Mutex::new(Registry::new()));

    // Start server
    let port = std::env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
    let ip = std::env::var("SERVER_PUBLIC_IP").unwrap_or("0.0.0.0".to_owned());
    let addr: SocketAddr = format!("{ip}:{port}").parse().unwrap();

    tracing::debug!("listening on {}", addr);

    generic_server_with_graceful_shutdown(addr, registry.clone())
        .await
        .map_err(|err| {
            tracing::error!("{err:?}");
            err
        })?;

    Ok(())
}

async fn generic_server_with_graceful_shutdown(
    addr: SocketAddr,
    registry: Arc<Mutex<Registry>>,
) -> Result<()> {
    // Load plugins and get the application router
    let (mut plugin_container, app_router) = app()?;

    // Add a `/health` route for health checks
    let health_router = Router::new().route("/health", get(health_check));

    // Add a `/metrics` route for Prometheus metrics
    let metrics_router = Router::new().route(
        "/metrics",
        get({
            let registry = Arc::clone(&registry);
            move || metrics_handler(registry)
        }),
    );

    // Combine the app router with the health and metrics routers
    let app_router = app_router.merge(health_router).merge(metrics_router);

    // Run the server
    Server::bind(&addr)
        .serve(
            app_router
                .layer(TraceLayer::new_for_http()) // Optional tracing middleware
                .into_make_service(),
        )
        .await
        .context("failed to start server")?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("shutting down gracefully");
            let _ = plugin_container.unload();
        }
    };

    Ok(())
}

/// Health check handler
async fn health_check() -> String {
    String::from("{\"status\": \"healthy\"}")
}

/// Expose Prometheus metrics
async fn metrics_handler(registry: Arc<Mutex<Registry>>) -> String {
    let registry = registry.lock().await;
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

fn config_tracing() {
    // Enable errors backtrace
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    use tracing::Level;
    use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

    let tracing_layer = tracing_subscriber::fmt::layer();
    let filter = filter::Targets::new()
        .with_target("hyper::proto", Level::INFO)
        .with_target("tower_http::trace", Level::DEBUG)
        .with_default(Level::DEBUG);

    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .init();
}
