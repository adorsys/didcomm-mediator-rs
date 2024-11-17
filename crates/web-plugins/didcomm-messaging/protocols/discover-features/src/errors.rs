use axum::Json;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("message body is malformed")]
    MalformedBody,
    #[error("No queries field in body")]
    QueryNotFound,
    #[error("query feature-type not supported try using `protocol`")]
    FeatureNOTSupported
}
impl DiscoveryError {
    /// Converts the error to an axum JSON representation.
    pub fn json(&self) -> Json<Value> {
        Json(json!({
            "error": self.to_string()
        }))
    }
}
