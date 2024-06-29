use generic_server::app;

use axum::Server;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
  
   //creating cancellation token which can be cloned and closed to tell server and process to finish
    let token = CancellationToken::new();
  
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable logging
    config_tracing();

    // Start server
    let port = std::env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    tracing::info!("listening on {addr}");

    // create a messager which will send the shutdown message to the server and its processes
    // any process which wishes to stop the server can send a shutdown message to the shutdown transmitter
    let (_shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<String>(2);

    // spawn task for server
     tokio::spawn(async move {
        Server::bind(&addr)
            .serve(app().into_make_service())
            .await
            .unwrap();
    });

   // watching on shutdown events/signals to gracefully shutdown servers
    tokio::select! {
        _msg = shutdown_rx.recv() => {eprintln!("\nshutting down gracefully:{}", _msg.unwrap()); token.cancel()}
        _ = tokio::signal::ctrl_c() => {eprintln!("\nshutting down gracefully"); token.cancel()}
    }
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
