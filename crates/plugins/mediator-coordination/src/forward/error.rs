use axum::Json;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("message body is malformed")]
    MalformedBody,
    #[error("Repository not set")]
    RepostitoryError
}
impl RoutingError {
    /// Converts the error to an axum JSON representation.
    pub fn json(&self) -> Json<Value> {
        Json(json!({
            "error": self.to_string()
        }))
    }
}

impl From<RoutingError> for Json<Value> {
    fn from(error: RoutingError) -> Self {
        error.json()
    }
}
