use axum::Server;
use generic_server::app;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Start server
    let port = std::env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    tracing::info!("listening on {addr}");
    generic_server_with_gracefull_shutdown(addr).await
}

async fn generic_server_with_gracefull_shutdown(addr: SocketAddr) {

    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable logging
    config_tracing();

    // Load plugins
    let (mut plugin_container, router) = app();

    // spawn task for server
    tokio::spawn(async move {
        Server::bind(&addr)
            .serve(router.into_make_service())
            .await
            .unwrap();
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
           tracing::info!("\nshuting down gracefully");
            let _ = plugin_container.unload();}
    };
}
fn config_tracing() {
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

#[cfg(test)]
mod tests {
    use super::generic_server_with_gracefull_shutdown;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_server_shutdown() {
        let port = std::env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
        tracing::info!("listening on {addr}");

        // run server in background
        tokio::spawn(generic_server_with_gracefull_shutdown(addr));
        // send a shutdown signal to main thread
    }
}
