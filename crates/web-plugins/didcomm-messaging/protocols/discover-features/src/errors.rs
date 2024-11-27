use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum DiscoveryError {
    #[error("message body is malformed")]
    MalformedBody,
    #[error("No queries field in body")]
    QueryNotFound,
    #[error("query feature-type not supported try using `protocol`")]
    FeatureNOTSupported,
}

impl IntoResponse for DiscoveryError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            DiscoveryError::MalformedBody => StatusCode::BAD_REQUEST,
            DiscoveryError::QueryNotFound => StatusCode::EXPECTATION_FAILED,
            DiscoveryError::FeatureNOTSupported => StatusCode::NOT_ACCEPTABLE,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status_code, body).into_response()
    }
}
