use didcomm_mediator::app;
use eyre::{Result, WrapErr};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow()?;

    // Enable logging
    config_tracing();

    // Configure server
    let port = std::env::var("SERVER_LOCAL_PORT").unwrap();

    let port = port.parse().context("failed to parse port")?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .context("failed to parse address")?;

    tracing::debug!("listening on {addr}");

    generic_server_with_graceful_shutdown(listener)
        .await
        .map_err(|err| {
            tracing::error!("{err:?}");
            err
        })?;

    Ok(())
}

async fn generic_server_with_graceful_shutdown(listener: TcpListener) -> Result<()> {
    // Load plugins
    let (mut plugin_container, router) = app()?;

    // Start server
    axum::serve(listener, router)
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
