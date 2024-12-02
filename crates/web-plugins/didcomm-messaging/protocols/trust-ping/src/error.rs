use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Error, PartialEq, Eq)]
pub(crate) enum TrustPingError {
    #[error("Missing sender DID")]
    MissingSenderDID,

    #[error("Malformed request. {0}")]
    MalformedRequest(String),
}

impl IntoResponse for TrustPingError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            TrustPingError::MissingSenderDID => StatusCode::BAD_REQUEST,
            TrustPingError::MalformedRequest(_) => StatusCode::BAD_REQUEST,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
