use super::OOBMessagesState;
use crate::models::{retrieve_or_generate_oob_inv, retrieve_or_generate_qr_image};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;

pub(crate) async fn handler_oob_inv(State(state): State<Arc<OOBMessagesState>>) -> Response {
    let mut store = state.store.lock().unwrap();
    let content = match retrieve_or_generate_oob_inv(
        &mut *store,
        &state.diddoc,
        &state.server_public_domain,
    ) {
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

    content.into_response()
}

pub(crate) async fn handler_oob_qr(State(state): State<Arc<OOBMessagesState>>) -> Response {
    let mut store = state.store.lock().unwrap();
    let oob_inv = match retrieve_or_generate_oob_inv(
        &mut *store,
        &state.diddoc,
        &state.server_public_domain,
    ) {
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

    let image_data = match retrieve_or_generate_qr_image(&mut *store, &oob_inv) {
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
    let mut store = state.store.lock().unwrap();
    let oob_inv = match retrieve_or_generate_oob_inv(
        &mut *store,
        &state.diddoc,
        &state.server_public_domain,
    ) {
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

    let image_data = match retrieve_or_generate_qr_image(&mut *store, &oob_inv) {
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
