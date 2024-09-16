mod pickup;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};
use didcomm::Message;
use hyper::{header::CONTENT_TYPE, StatusCode};
use std::sync::Arc;

use crate::constants::DIDCOMM_ENCRYPTED_MIME_TYPE;
use mediator_coordination::web::{pack_response_message, AppState};

#[axum::debug_handler]
pub(crate) fn handle_message_pickup(
    State(state): State<Arc<AppState>>,
    Extension(message): Extension<Message>,
) -> Response {
    // handle mediation request
    let delegate_response = match message.type_.as_str() {
        KEYLIST_UPDATE_2_0 => {
            web::coord::handler::stateful::process_plain_keylist_update_message(
                Arc::clone(&state),
                message,
            )
            .await
        }
        MEDIATE_REQUEST_2_0 => {
            web::coord::handler::stateful::process_mediate_request(&state, &message).await
        }
        _ => {
            let response = (
                StatusCode::BAD_REQUEST,
                MediationError::UnsupportedOperation.json(),
            );

            return response.into_response();
        }
    };

    process_response(state, delegate_response).await
}

async fn process_response(state: Arc<AppState>, response: Result<Message, Response>) -> Response {
    match response {
        Ok(message) => {
            pack_response_message(&message, &state.did_resolver, &state.secrets_resolver)
                .await
                .map(|packed| {
                    (
                        StatusCode::ACCEPTED,
                        [(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)],
                        Json(packed),
                    )
                        .into_response()
                })
                .unwrap_or_else(|err| err.into_response())
        }
        Err(response) => response,
    }
}
