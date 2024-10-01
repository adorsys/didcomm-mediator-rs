use axum::Json;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RotationError {
    #[error("Could not deserialize from prior")]
    DeserializationError,
    #[error("Could not rotate did unknown issuer")]
    RotationError,
    #[error("Invalid jwt signature on FromPrior value")]
    InvalidSignature,
    #[error("could not unpack fromprior")]
    InvalidFromPrior

}
impl RotationError {
    /// Converts the error to an axum JSON representation.
    pub fn json(&self) -> Json<Value> {
        Json(json!({
            "error": self.to_string()
        }))
    }
}

impl From<RotationError> for Json<Value> {
    fn from(error: RotationError) -> Self {
        error.json()
    }
}
