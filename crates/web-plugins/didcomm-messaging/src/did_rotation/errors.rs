use axum::Json;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum RotationError {
    #[error("Could not rotate did unknown issuer")]
    UnknownIssuer,
    #[error("could not unpack fromprior")]
    InvalidFromPrior,
    #[error("Internal server error")]
    InternalServerError,
}

impl RotationError {
    /// Converts the error to an axum JSON representation.
    pub(crate) fn json(&self) -> Json<Value> {
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
