use serde_json::Error as SerdeError;

use crate::{
    crypto::{
        ed25519::Ed25519KeyPair,
        traits::{Error as CryptoError, KeyMaterial},
    }
};

#[derive(Debug)]
pub enum DIDPeerMethodError {
    CryptoError(CryptoError),
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
