use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Registry for error types found across the DID core specification,
/// and especially during the DID resolution process.
///
/// See https://www.w3.org/TR/did-spec-registries/#error
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Error)]
#[serde(rename_all = "camelCase")]
pub enum DIDResolutionError {
    #[error("invalidDid")]
    InvalidDid,
    #[error("invalidDidUrl")]
    InvalidDidUrl,
    #[error("notFound")]
    NotFound,
    #[error("representationNotSupported")]
    RepresentationNotSupported,
    #[error("methodNotSupported")]
    MethodNotSupported,
    #[error("internalError")]
    InternalError,
    #[error("invalidPublicKey")]
    InvalidPublicKey,
    #[error("invalidPublicKeyLength")]
    InvalidPublicKeyLength,
    #[error("invalidPublicKeyType")]
    InvalidPublicKeyType,
    #[error("unsupportedPublicKeyType")]
    UnsupportedPublicKeyType,
    #[error("notAllowedVerificationMethodType")]
    NotAllowedVerificationMethodType,
    #[error("notAllowedKeyType")]
    NotAllowedKeyType,
    #[error("notAllowedMethod")]
    NotAllowedMethod,
    #[error("notAllowedCertificate")]
    NotAllowedCertificate,
    #[error("notAllowedLocalDuplicateKey")]
    NotAllowedLocalDuplicateKey,
    #[error("notAllowedLocalDerivedKey")]
    NotAllowedLocalDerivedKey,
    #[error("notAllowedGlobalDuplicateKey")]
    NotAllowedGlobalDuplicateKey,
}
