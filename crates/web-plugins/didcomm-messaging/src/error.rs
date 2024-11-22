use axum::Json;
use didcomm::error::ErrorKind as DidcommErrorKind;
use serde_json::{json, Value};
use thiserror::Error;

/// Represents errors that can occur during mediation.
#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("message must not be anoncrypt'd")]
    AnonymousPacker,
    #[error("assumed didcomm-encrypted message is malformed")]
    MalformedDidcommEncrypted,
    #[error("could not unpack message")]
    MessageUnpackingFailure,
    #[error("could not pack message: {0}")]
    MessagePackingFailure(DidcommErrorKind),
    #[error("unsupported content-type, only accept application/didcomm-encrypted+json")]
    NotDidcommEncryptedPayload,
    #[error("unparseable payload")]
    UnparseablePayload,
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
