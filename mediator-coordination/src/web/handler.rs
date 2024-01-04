use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Extension,
};
use didcomm::Message;
use hyper::StatusCode;
use std::sync::Arc;

use crate::{
    constant::KEYLIST_UPDATE_2_0,
    web::{self, error::MediationError, AppState},
};

#[axum::debug_handler]
pub async fn process_didcomm_message(
    State(state): State<Arc<AppState>>,
    Extension(message): Extension<Message>,
    _headers: HeaderMap,
) -> Response {
    let msg = serde_json::to_string_pretty(&message).unwrap();
    tracing::info!("{msg}");

    match message.type_.as_str() {
        KEYLIST_UPDATE_2_0 => {
            web::coord::handler::stateful::process_plain_keylist_update_message(state, message)
                .await
        }
        _ => {
            let response = (
                StatusCode::BAD_REQUEST,
                MediationError::NoReturnRouteAllDecoration.json(),
            );

            return response.into_response();
        }
    }
}
