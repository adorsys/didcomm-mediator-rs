use axum::{
    http::{Method, Uri},
    middleware,
    response::{IntoResponse, Response},
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
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

#[tokio::main]
async fn main() -> Result<()> {
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

    // spawning task tracker on server to handle server's shutdown
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

    Ok(())
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
