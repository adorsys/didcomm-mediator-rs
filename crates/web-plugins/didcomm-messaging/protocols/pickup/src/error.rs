use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub(crate) enum PickupError {
    #[error("Missing sender DID")]
    MissingSenderDID,

    #[error("{0}")]
    InternalError(String),

    #[error("No client connection found")]
    MissingClientConnection,

    #[error("Malformed request. {0}")]
    MalformedRequest(String),

    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl IntoResponse for PickupError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            PickupError::MissingSenderDID | PickupError::MalformedRequest(_) => {
                StatusCode::BAD_REQUEST
            }
            PickupError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PickupError::MissingClientConnection => StatusCode::UNAUTHORIZED,
            PickupError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
