use serde_json::Error as SerdeError;

use crate::{crypto::traits::Error as CryptoError, methods::errors::DIDResolutionError};

#[derive(Debug)]
pub enum DIDPeerMethodError {
    CryptoError(CryptoError),
    DIDResolutionError(DIDResolutionError),
    EmptyArguments,
    IllegalArgument,
    InvalidHash,
    InvalidPeerDID,
    InvalidPurposeCode,
    InvalidStoredVariant,
    MalformedPeerDID,
    MalformedLongPeerDID,
    SerdeError(SerdeError),
    UnexpectedPurpose,
    UnsupportedPeerDIDAlgorithm,
}

impl From<CryptoError> for DIDPeerMethodError {
    fn from(err: CryptoError) -> Self {
        Self::CryptoError(err)
    }
}

impl From<DIDResolutionError> for DIDPeerMethodError {
    fn from(err: DIDResolutionError) -> Self {
        Self::DIDResolutionError(err)
    }
}

impl From<SerdeError> for DIDPeerMethodError {
    fn from(err: SerdeError) -> Self {
        Self::SerdeError(err)
    }
}
