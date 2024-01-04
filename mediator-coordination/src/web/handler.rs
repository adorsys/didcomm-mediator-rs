use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};
use didcomm::Message;
use hyper::{header::CONTENT_TYPE, StatusCode};
use std::sync::Arc;

use crate::{
    constant::{DIDCOMM_ENCRYPTED_MIME_TYPE, KEYLIST_UPDATE_2_0},
    web::{self, error::MediationError, AppState},
};

#[axum::debug_handler]
pub async fn process_didcomm_message(
    State(state): State<Arc<AppState>>,
    Extension(message): Extension<Message>,
) -> Response {
    let msg = serde_json::to_string_pretty(&message).unwrap();
    tracing::info!("request: {msg}");

    let delegate_response = match message.type_.as_str() {
        KEYLIST_UPDATE_2_0 => {
            web::coord::handler::stateful::process_plain_keylist_update_message(
                Arc::clone(&state),
                message,
            )
            .await
        }
        _ => {
            let response = (
                StatusCode::BAD_REQUEST,
                MediationError::NoReturnRouteAllDecoration.json(),
            );

            return response.into_response();
        }
    };

    process_response_from_delegate_handler(state, delegate_response).await
}

async fn process_response_from_delegate_handler(
    state: Arc<AppState>,
    response: Result<Message, Response>,
) -> Response {
    // Extract plain message or early return error response
    let plain_response_message = match response {
        Ok(message) => message,
        Err(response) => return response,
    };

    let msg = serde_json::to_string_pretty(&plain_response_message).unwrap();
    tracing::info!("response: {msg}");

    // Pack response message
    let packed_message = match web::midlw::pack_response_message(
        &plain_response_message,
        &state.did_resolver,
        &state.secrets_resolver,
    )
    .await
    {
        Ok(packed) => packed,
        Err(response) => return response,
    };

    // Build final response
    let response = (
        StatusCode::ACCEPTED,
        [(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)],
        Json(packed_message),
    );

    response.into_response()
}
