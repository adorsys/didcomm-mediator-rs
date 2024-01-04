use std::sync::Arc;

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::web::AppState;

#[axum::debug_handler]
pub async fn process_didcomm_message(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: String,
) -> Response {
    "".into_response()
}
