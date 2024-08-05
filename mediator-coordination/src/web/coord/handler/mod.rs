pub mod stateful;
#[cfg(feature = "stateless")]
mod stateless;

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

use crate::web::AppState;
use crate::{
    constant::DIDCOMM_ENCRYPTED_MIME_TYPE,
    model::coord::MediationRequest,
    web::coord::midlw::{self, *},
};

#[axum::debug_handler]
pub async fn process_didcomm_mediation_request_message(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: String,
) -> Response {
    // Enforce request content type to match `didcomm-encrypted+json`
    midlw::run!(ensure_content_type_is_didcomm_encrypted(&headers));

    // Unpack payload message
    let plain_message = midlw::run!(
        unpack_request_message(&payload, &state.did_resolver, &state.secrets_resolver).await
    );

    // Attempt to parse message body into a mediation request
    let mediation_request = midlw::run!(parse_message_body_into_mediation_request(&plain_message));

    // Handle mediation request with its matching protocol
    let plain_response_message = match mediation_request {
        #[cfg(feature = "stateless")]
        MediationRequest::Stateless(req) => midlw::run!(
            stateless::process_plain_mediation_request_over_dics(&state, &plain_message, &req)
                .await
        ),
        #[cfg(feature = "stateful")]
        MediationRequest::Stateful(_req) => {
            midlw::run!(stateful::process_mediate_request(&state, &plain_message).await)
        }
    };

    // Pack response message
    let packed_message = midlw::run!(
        pack_response_message(
            &plain_response_message,
            &state.did_resolver,
            &state.secrets_resolver
        )
        .await
    );

    // Build final response
    let response = (
        StatusCode::ACCEPTED,
        [(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)],
        Json(packed_message),
    );

    response.into_response()
}
