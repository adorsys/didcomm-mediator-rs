use axum::Json;
use serde_json::{json, Value};
use thiserror::Error;

/// Represents errors that can occur during mediation.
#[derive(Debug, Error)]
pub enum MediationError {
    #[error("message must not be anoncrypt'd")]
    AnonymousPacker,
    #[error("anti spam check failure")]
    AntiSpamCheckFailure,
    #[error("could not parse into mediate request")]
    InvalidMediationRequestFormat,
    #[error("invalid message type")]
    InvalidMessageType,
    #[error("assumed didcomm-encrypted message is malformed")]
    MalformedDidcommEncrypted,
    #[error("could not unpack message")]
    MessageUnpackingFailure,
    #[error("could not pack message")]
    MessagePackingFailure,
    #[error("unsupported content-type, only accept application/didcomm-encrypted+json")]
    NotDidcommEncryptedPayload,
    #[error("unsupported did method")]
    UnsupportedDidMethod,
}

impl MediationError {
    /// Converts the error to an axum JSON representation.
    pub fn json(&self) -> Json<Value> {
        Json(json!({
            "error": self.to_string()
        }))
    }
}

impl From<MediationError> for Json<Value> {
    fn from(error: MediationError) -> Self {
        error.json()
    }
}
