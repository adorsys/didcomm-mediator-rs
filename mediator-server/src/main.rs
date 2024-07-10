use std::net::SocketAddr;

use axum::{response::{IntoResponse, Response}, Router, middleware, Json, http::{Uri, Method}};
use serde_json::json;
use uuid::Uuid;

mod coordinate_mediation;
mod ctx;
mod constants;
mod error;
mod models;
mod log;
mod utils;

use crate::{models::RecipientController, log::log_request};

pub use self::{error::{Error, Result, ClientError}, ctx::Ctx};

#[tokio::main]
async fn main() -> Result<()> {
    let mc = RecipientController::new().await;
    let routes_mediate_request = coordinate_mediation::routes_mediate_request::routes(mc.clone());
    let routes_all = Router::new()
        .nest("/coordinate-mediation/2.0/mediate-request", routes_mediate_request)
        .layer(middleware::map_response(main_response_mapper));

    // region: --- Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("->> Listening on {addr}\n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    // endregion: --- Start Server

    Ok(())
}

async fn main_response_mapper(
    uri: Uri,
    req_method: Method,
    res:Response) -> Response {
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
