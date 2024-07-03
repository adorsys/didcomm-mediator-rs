use axum::Server;
use generic_server::{app, unload_for_shutdown};
use once_cell::sync::Lazy;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_util::sync::CancellationToken;

// create a global shutdown signal transmitter
static SHUTDOWN: Lazy<Arc<Mutex<Option<Sender<String>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));
#[tokio::main]
async fn main() {
    // Start server
    let port = std::env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    tracing::info!("listening on {addr}");
    run_and_shutdown_server(addr).await
}

async fn run_and_shutdown_server(add: SocketAddr) {
    //creating cancellation token which can be cloned and closed to tell server and process to finish
    let token = CancellationToken::new();

    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable logging
    config_tracing();
    // create a messager which will send the shutdown message to the server and its processes
    // any process which wishes to stop the server can send a shutdown message to the shutdown transmitter
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<String>(2);

    // spawn task for server
    tokio::spawn(async move {
        Server::bind(&add)
            .serve(app().into_make_service())
            .await
            .unwrap();
    });
    // watching on shutdown events/signals to gracefully shutdown servers
    let mut lock = SHUTDOWN.lock().await;
    lock.replace(shutdown_tx);

    tokio::select! {
        _msg = shutdown_rx.recv() => {eprintln!("\nUmounting plugins\nshutting down gracefully:{:?}", _msg); unload_for_shutdown(); token.cancel(); }
        _ = tokio::signal::ctrl_c() => {eprintln!("\nUnmounting Plugins\nshutting down gracefully"); unload_for_shutdown(); token.cancel(); }
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
    use super::{SHUTDOWN, run_and_shutdown_server};
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_server_shutdown() {
        let port = std::env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
        tracing::info!("listening on {addr}");

        // run server in background
        tokio::spawn(run_and_shutdown_server(addr));

        // send shutdown signal
        let mut lock = SHUTDOWN.lock().await;
        let sender = lock.as_mut();
        match sender {
            Some(sender) => {
                sender.send("value".to_owned()).await.unwrap();
            }
            None => {}
        }
        
    }
}
