use axum::{http::StatusCode, response::IntoResponse, Json};
use database::RepositoryError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum PickupError {
    #[error("Sender DID is missing in the message")]
    MissingSenderDID,

    #[error("{0}")]
    InternalError(String),

    #[error("No client connection found")]
    MissingClientConnection,

    #[error("Malformed request. {0}")]
    MalformedRequest(String),

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("internal server error")]
    RepositoryError { inner: RepositoryError },
}

impl PartialEq for PickupError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MissingSenderDID, Self::MissingSenderDID) => true,
            (Self::InternalError(a), Self::InternalError(b)) => a == b,
            (Self::MissingClientConnection, Self::MissingClientConnection) => true,
            (Self::MalformedRequest(a), Self::MalformedRequest(b)) => a == b,
            (Self::ServiceUnavailable, Self::ServiceUnavailable) => true,
            _ => false,
        }
    }
}

impl IntoResponse for PickupError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            PickupError::MissingSenderDID | PickupError::MalformedRequest(_) => {
                StatusCode::BAD_REQUEST
            }
            PickupError::InternalError(_) | PickupError::RepositoryError { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            PickupError::MissingClientConnection => StatusCode::UNAUTHORIZED,
            PickupError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        };

        let body = match &self {
            PickupError::RepositoryError { inner } => {
                tracing::error!("Repository error: {:?}", inner);
                Json(serde_json::json!({ "error": "Internal server error" }))
            }
            _ => Json(serde_json::json!({ "error": self.to_string() })),
        };

        (status_code, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn repository_error_response_body_is_opaque() {
        let db_error = "DB connection failed: MongoServerError: command { find } on namespace admin.system.namespaces failed: Database 'secret_db' does not exist".to_string();
        let err: PickupError = PickupError::RepositoryError {
            inner: RepositoryError::Generic(db_error),
        };
        let response = err.into_response();

        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Repository errors should return 500"
        );

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(
            json.get("error").and_then(|v| v.as_str()),
            Some("Internal server error"),
            "Error message should be opaque, not raw DB text"
        );
        assert!(
            !body_str.contains("secret_db")
                && !body_str.contains("MongoServerError")
                && !body_str.contains("system.namespaces")
                && !body_str.contains("Database"),
            "Response body should NOT contain DB/schema details"
        );
    }
}
