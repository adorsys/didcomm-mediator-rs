use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Error)]
pub(crate) enum TrustPingError<'a> {
    #[error("Missing sender DID")]
    MissingSenderDID,

    #[error("Malformed request. {0}")]
    MalformedRequest(&'a str),
}

impl IntoResponse for TrustPingError<'_> {
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
