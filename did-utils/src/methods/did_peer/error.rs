use serde_json::Error as SerdeError;

use crate::crypto::traits::Error as CryptoError;

#[derive(Debug)]
pub enum DIDPeerMethodError {
    CryptoError(CryptoError),
    EmptyArguments,
    UnexpectedPurpose,
    InvalidPurposeCode,
    InvalidStoredVariant,
    SerdeError(SerdeError),
}

impl From<CryptoError> for DIDPeerMethodError {
    fn from(err: CryptoError) -> Self {
        Self::CryptoError(err)
    }
}

impl From<SerdeError> for DIDPeerMethodError {
    fn from(err: SerdeError) -> Self {
        Self::SerdeError(err)
    }
}
