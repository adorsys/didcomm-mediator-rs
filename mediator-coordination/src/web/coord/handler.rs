use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::web::AppState;

#[axum::debug_handler]
pub async fn process_didcomm_mediation_request_message(
    State(state): State<AppState>,
    _headers: HeaderMap,
    _message: String,
) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "diddoc": state.diddoc
        })),
    )
        .into_response()
}
