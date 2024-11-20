use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForwardError {
    #[error("message body is malformed")]
    MalformedBody,
    #[error("Uncoordinated sender")]
    UncoordinatedSender,
    #[error("Internal server error")]
    InternalServerError,
}

impl IntoResponse for ForwardError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            ForwardError::MalformedBody => StatusCode::BAD_REQUEST,
            ForwardError::UncoordinatedSender => StatusCode::UNAUTHORIZED,
            ForwardError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
