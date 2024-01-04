use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Extension,
};
use didcomm::Message;
use std::sync::Arc;

use crate::web::AppState;

#[axum::debug_handler]
pub async fn process_didcomm_message(
    State(_state): State<Arc<AppState>>,
    Extension(message): Extension<Message>,
    _headers: HeaderMap,
) -> Response {
    serde_json::to_string_pretty(&message)
        .unwrap()
        .to_string()
        .into_response()
}
