use thiserror::Error;

use crate::web::error;

#[derive(Debug, Error)]
pub enum RotationError {
    #[error("Could not deserialize from prior")]
    DeserializationError,
    #[error("Could not rotate did unknown issuer")]
    RotationError
}