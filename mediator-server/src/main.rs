use axum::{
    http::{Method, Uri},
    middleware,
    response::{IntoResponse, Response},
    Json, Router,
};
use once_cell::sync::Lazy;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

mod constants;
mod coordinate_mediation;
mod ctx;
mod error;
mod log;
mod models;
mod utils;

use crate::{log::log_request, models::RecipientController};

pub use self::{
    ctx::Ctx,
    error::{ClientError, Error, Result},
};
// create a global signal shutdown signal transmitter
static SHUTDOWN: Lazy<Arc<Mutex<Option<Sender<String>>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

#[tokio::main]
async fn main() -> Result<()> {
    run_shutdown().await;
    Ok(())
}
    async fn run_shutdown() {
    let mc = RecipientController::new().await;
    let routes_mediate_request = coordinate_mediation::routes_mediate_request::routes(mc.clone());
    let routes_all = Router::new()
        .nest(
            "/coordinate-mediation/2.0/mediate-request",
            routes_mediate_request,
        )
        .layer(middleware::map_response(main_response_mapper));

    // create a messager which will send the shutdown message to the server and its processes
    // any process which wishes to stop the server can send a shutdown message to the shutdown transmitter
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<String>(2);

    // create cancellation tokens which when closed will tell processes to shutdown
    let token = CancellationToken::new();

    // region: --- Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("->> Listening on {addr}\n");

// initialising global shutdown transmitter
 let mut lock = SHUTDOWN.lock().await;
 lock.replace(shutdown_tx);

    // spawning task on server to handle server's shutdown
    tokio::spawn(async move {
        axum::Server::bind(&addr)
            .serve(routes_all.into_make_service())
            .await
            .unwrap();
    });
    // endregion: --- Start Server

    // gracyfully shutting down the server on CTRL-C or on shutdown alert from shutdown transmitter
    // select on the operations for which we wish to gracefully shutdown the server
    tokio::select! {
            _shutdown_message = shutdown_rx.recv() => {eprintln!("shutting down"); token.cancel()},
            _ = tokio::signal::ctrl_c() => {eprintln!("shutting down"); token.cancel()},
    }
}

async fn main_response_mapper(uri: Uri, req_method: Method, res: Response) -> Response {
    println!("->> {:<12} - main_response_mapper", "RESPONSE_MAPPER");
    let uuid = Uuid::new_v4();

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_satus_and_error());

    // -- If client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });
            println!("    ->> client_error_body: {client_error_body}");

            // Build the new response from the client_error_body.
            (*status_code, Json(client_error_body)).into_response()
        });

    // -- Build and log the server log line
    let client_error = client_status_error.unzip().1;
    let _ = log_request(uuid, req_method, uri, service_error, client_error).await;

    println!();

    error_response.unwrap_or(res)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_shutdown_with_shutdown_signal() {
      
        // run server in background
        tokio::spawn(run_shutdown());

        // send shutdown signal
        let mut lock = SHUTDOWN.lock().await;
        let sender = lock.as_mut();
        match sender {
            Some(sender) => {
                sender.send("Shutdown".to_owned()).await.unwrap();

            }
            None => {}
        }

    }
}