use std::{error::Error, sync::Arc};

use super::OOBMessagesState;
use crate::models::{retrieve_or_generate_oob_inv, retrieve_or_generate_qr_image, OobMessage};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use multibase::Base::Base64Url;
use serde_json::json;

pub(crate) async fn handler_oob_inv(State(state): State<Arc<OOBMessagesState>>) -> Response {
    let mut fs = state.filesystem.lock().unwrap();
    let (server_public_domain, storage_dirpath) = match get_environment_variables() {
        Ok(result) => result,
        Err(err) => {
            tracing::error!("Error: {err:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let html_content =
        match retrieve_or_generate_oob_inv(&mut *fs, &server_public_domain, &storage_dirpath) {
            Ok(oob_inv) => oob_inv,
            Err(err) => {
                tracing::error!("Failed to retrieve or generate oob invitation: {err:?}");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Could not process request at this time. Please try again later",
                )
                    .into_response();
            }
        };

    let html_content = format!(
        r#"
        <!DOCTYPE html>
        
        <html lang="en">
            <style>
                pre {{
                    white-space: pre-wrap;
                    word-wrap: break-word;
                }}
            </style>
        <body>
       <a href={}>out of band invitation url</a>
        </body>
        </html>
            "#,
        html_content
    );

    Html(html_content).into_response()
}
pub(crate) async fn decode_oob_inv(State(state): State<Arc<OOBMessagesState>>) -> Response {
    let encoded_inv = &state.oobmessage;
    let encoded_inv: Vec<&str> = encoded_inv.split("oob").collect();
    let encoded_inv = encoded_inv.get(1).unwrap();
    let decoded_inv = Base64Url.decode(encoded_inv).unwrap_or_default();
    let oobmessage: OobMessage = serde_json::from_slice(&decoded_inv).unwrap_or_default();
    let view = json!(oobmessage);
    let html_content = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
            <style>
                pre {{
                    white-space: pre-wrap;
                    word-wrap: break-word;
                }}
            </style>
        <body>
        <pre>
            {}
        </pre>
        </body>
        </html>
            "#,
        view
    );
    Html(html_content).into_response()
}

pub(crate) async fn handler_oob_qr(State(state): State<Arc<OOBMessagesState>>) -> Response {
    let mut fs = state.filesystem.lock().unwrap();
    let (server_public_domain, storage_dirpath) = match get_environment_variables() {
        Ok(result) => result,
        Err(err) => {
            tracing::error!("Error: {err:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let oob_inv =
        match retrieve_or_generate_oob_inv(&mut *fs, &server_public_domain, &storage_dirpath) {
            Ok(oob_inv) => oob_inv,
            Err(err) => {
                tracing::error!("Failed to retrieve or generate oob invitation: {err:?}");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Could not process request at this time. Please try again later",
                )
                    .into_response();
            }
        };

    let image_data = match retrieve_or_generate_qr_image(&mut *fs, &storage_dirpath, &oob_inv) {
        Ok(data) => data,
        Err(err) => {
            tracing::error!("Failed to retrieve or generate QR code image: {err:?}");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Could not process request at this time. Please try again later",
            )
                .into_response();
        }
    };

    let html_content = format!(
        r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>QR Code Image</title>
                </head>
                <body>
                    <img src="data:image/png;base64,{}" alt="QR Code Image">
                </body>
            </html>
            "#,
        &image_data
    );

    (
        [(header::CONTENT_SECURITY_POLICY, "img-src 'self' data:")],
        Html(html_content),
    )
        .into_response()
}

pub(crate) async fn handler_landing_page_oob(
    State(state): State<Arc<OOBMessagesState>>,
) -> Response {
    let mut fs = state.filesystem.lock().unwrap();
    let (server_public_domain, storage_dirpath) = match get_environment_variables() {
        Ok(result) => result,
        Err(err) => {
            tracing::error!("Error: {err:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let oob_inv =
        match retrieve_or_generate_oob_inv(&mut *fs, &server_public_domain, &storage_dirpath) {
            Ok(oob_inv) => oob_inv,
            Err(err) => {
                tracing::error!("Failed to retrieve or generate oob invitation: {err:?}");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Could not process request at this time. Please try again later",
                )
                    .into_response();
            }
        };

    let image_data = match retrieve_or_generate_qr_image(&mut *fs, &storage_dirpath, &oob_inv) {
        Ok(data) => data,
        Err(err) => {
            tracing::error!("Failed to retrieve or generate QR code image: {err:?}");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Could not process request at this time. Please try again later",
            )
                .into_response();
        }
    };

    let html_content = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>didcomm-mediator-rs</title>
        </head>
        <div>
            <body>
                    <div style="text-align:center">
                        <h1>&nbsp;didcomm-mediator-rs</h1>
                    </div>
                    <div style="text-align:center">
                        <p>DIDComm v2 mediator</p></p><br />
                    </div>
                    <div style="text-align:center">
                        <h3>Scan the QR code invitation shown below to start a mediation request:</h3>
                        <img src="data:image/png;base64,{}" alt="QR Code Image" width="300">
                        <h3>Or just copy and paste the following Out of Band invitation URL:</h3>
                        <iframe src="/oob_url" title="OOB URL" height="200" width="500" frameBorder="0"></iframe>
                    </div>
            </body>
        </div>
    </html>
        "#,
        &image_data,
    );

    (
        [(header::CONTENT_SECURITY_POLICY, "img-src 'self' data:")],
        Html(html_content),
    )
        .into_response()
}

fn get_environment_variables() -> Result<(String, String), Box<dyn Error>> {
    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN")
        .map_err(|_| "SERVER_PUBLIC_DOMAIN env variable required")?;

    let storage_dirpath =
        std::env::var("STORAGE_DIRPATH").map_err(|_| "STORAGE_DIRPATH env variable required")?;

    Ok((server_public_domain, storage_dirpath))
}
