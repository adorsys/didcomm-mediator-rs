use axum::Json;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("message body is malformed")]
    MalformedBody,
    #[error("Repository not set")]
    RepostitoryError,
    #[error("No query field in body")]
    QueryNotFound
}
impl DiscoveryError {
    /// Converts the error to an axum JSON representation.
    pub fn json(&self) -> Json<Value> {
        Json(json!({
            "error": self.to_string()
        }))
    }
}

impl From<DiscoveryError> for Json<Value> {
    fn from(error: DiscoveryError) -> Self {
        error.json()
    }
}
