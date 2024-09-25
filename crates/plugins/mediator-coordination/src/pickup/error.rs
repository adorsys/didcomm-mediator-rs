use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Error)]
pub enum PickupError {
    #[error("Missing sender DID")]
    MissingSenderDID,

    #[error("Missing persistence layer")]
    MissingRepository,

    #[error("No client connection found")]
    MissingClientConnection,

    #[error("Database error")]
    DatabaseError,

    #[error("Malformed request: {0}")]
    MalformedRequest(String),
}

impl IntoResponse for PickupError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            PickupError::MissingSenderDID => StatusCode::BAD_REQUEST,
            PickupError::MissingRepository => StatusCode::INTERNAL_SERVER_ERROR,
            PickupError::MissingClientConnection => StatusCode::UNAUTHORIZED,
            PickupError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PickupError::MalformedRequest(_) => StatusCode::BAD_REQUEST,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
