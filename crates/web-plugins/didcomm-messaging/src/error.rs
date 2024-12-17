use axum::Json;
use serde_json::{json, Value};
use thiserror::Error;

/// Represents errors that can occur during mediation.
#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("message must not be anoncrypt'd")]
    AnonymousPacker,
    #[error("assumed didcomm-encrypted message is malformed")]
    MalformedDidcommEncrypted,
    #[error("Internal server error")]
    InternalServer,
    #[error("unsupported content-type, only accept application/didcomm-encrypted+json")]
    NotDidcommEncryptedPayload,
}

impl Error {
    /// Converts the error to an axum JSON representation.
    pub fn json(&self) -> Json<Value> {
        Json(json!({
            "error": self.to_string()
        }))
    }
}

impl From<Error> for Json<Value> {
    fn from(error: Error) -> Self {
        error.json()
    }
}
