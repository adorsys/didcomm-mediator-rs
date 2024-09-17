use serde::Serialize;
use database::RepositoryError;
use thiserror::Error;
use axum::{http::StatusCode, response::IntoResponse, Json};


#[derive(Debug, Serialize, Error)]
pub enum PickupError {
    #[error("Missing sender DID")]
    MissingSenderDID,

    #[error("Missing persistence layer")]
    MissingRepository,

    #[error("No client connection found")]
    MissingClientConnection,

    #[error("Database error: {0}")]
    RepositoryError(#[from] RepositoryError),
}

impl IntoResponse for PickupError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            PickupError::MissingSenderDID => StatusCode::BAD_REQUEST,
            PickupError::MissingRepository => StatusCode::INTERNAL_SERVER_ERROR,
            PickupError::MissingClientConnection => StatusCode::INTERNAL_SERVER_ERROR,
            PickupError::RepositoryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
