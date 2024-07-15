use serde_json::Error as SerdeError;

use crate::{crypto::errors::Error as CryptoError, methods::errors::DIDResolutionError};

#[derive(Debug)]
pub enum DIDPeerMethodError {
    CryptoError(CryptoError),
    DIDParseError,
    DIDResolutionError(DIDResolutionError),
    EmptyArguments,
    IllegalArgument,
    InvalidHash,
    InvalidPeerDID,
    InvalidPurposeCode,
    InvalidStoredVariant,
    MalformedPeerDID,
    MalformedLongPeerDID,
    RegexMismatch,
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

impl From<DIDPeerMethodError> for DIDResolutionError {
    fn from(err: DIDPeerMethodError) -> Self {
        use DIDPeerMethodError::*;

        match err {
            CryptoError(_) => Self::InvalidDid,
            DIDParseError => Self::InvalidDid,
            DIDResolutionError(err) => err,
            EmptyArguments => Self::InternalError,
            IllegalArgument => Self::InvalidDid,
            InvalidHash => Self::InvalidDid,
            InvalidPeerDID => Self::InvalidDid,
            InvalidPurposeCode => Self::InvalidDid,
            InvalidStoredVariant => Self::InternalError,
            MalformedPeerDID => Self::InvalidDid,
            MalformedLongPeerDID => Self::InvalidDid,
            RegexMismatch => Self::InvalidDid,
            SerdeError(_) => Self::InvalidDid,
            UnexpectedPurpose => Self::InvalidDid,
            UnsupportedPeerDIDAlgorithm => Self::MethodNotSupported,
        }
    }
}
