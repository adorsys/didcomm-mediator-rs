use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use thiserror::Error;

/// Represents errors that can occur during mediation.
#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum MediationError {
    #[error("Sender DID is missing in the message")]
    MissingSenderDID,
    #[error("No return route all decoration")]
    NoReturnRouteAllDecoration,
    #[error("invalid message type")]
    InvalidMessageType,
    #[error("uncoordinated sender")]
    UncoordinatedSender,
    #[error("could not parse into expected message format")]
    UnexpectedMessageFormat,
    #[error("internal server error")]
    InternalServerError,
    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl IntoResponse for MediationError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            MediationError::NoReturnRouteAllDecoration
            | MediationError::InvalidMessageType
            | MediationError::MissingSenderDID => StatusCode::BAD_REQUEST,
            MediationError::UncoordinatedSender => StatusCode::UNAUTHORIZED,
            MediationError::UnexpectedMessageFormat => StatusCode::BAD_REQUEST,
            MediationError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            MediationError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
