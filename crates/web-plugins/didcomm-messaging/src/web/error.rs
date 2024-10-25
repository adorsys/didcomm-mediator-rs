use axum::Json;
use didcomm::error::ErrorKind as DidcommErrorKind;
use serde_json::{json, Value};
use thiserror::Error;

/// Represents errors that can occur during mediation.
#[derive(Debug, Error)]
pub enum MediationError {
    #[error("message must not be anoncrypt'd")]
    AnonymousPacker,
    #[error("anti spam check failure")]
    AntiSpamCheckFailure,
    #[error("duplicate command")]
    DuplicateCommand,
    #[error("generic: {0}")]
    Generic(String),
    #[error("invalid message type")]
    InvalidMessageType,
    #[error("assumed didcomm-encrypted message is malformed")]
    MalformedDidcommEncrypted,
    #[error("could not unpack message")]
    MessageUnpackingFailure,
    #[error("could not pack message: {0}")]
    MessagePackingFailure(DidcommErrorKind),
    #[error("message must be decorated with return route all extension")]
    NoReturnRouteAllDecoration,
    #[error("unsupported content-type, only accept application/didcomm-encrypted+json")]
    NotDidcommEncryptedPayload,
    #[error("uncoordinated sender")]
    UncoordinatedSender,
    #[error("could not parse into expected message format")]
    UnexpectedMessageFormat,
    #[error("unparseable payload")]
    UnparseablePayload,
    #[error("unsupported did method")]
    UnsupportedDidMethod,
    #[error("unsupported operation")]
    UnsupportedOperation,
    #[error("Could not store Message")]
    PersisenceError,
    #[error("Could not deserialize Message")]
    DeserializationError,
    #[error("Repository not set")]
    RepostitoryError,
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
