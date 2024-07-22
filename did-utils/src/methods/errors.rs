use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Registry for [error] types found across the DID core specification,
/// and especially during the DID resolution process.
///
/// [error]: https://www.w3.org/TR/did-spec-registries/#error
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Error)]
#[serde(rename_all = "camelCase")]
pub enum DIDResolutionError {
    #[error("invalidDid")]
    InvalidDid,
    #[error("invalidDidUrl")]
    InvalidDidUrl,
    #[error("invalidDidUrlPrefix")]
    InvalidDidUrlPrefix,
    #[error("invalidDidUrlFormat")]
    InvalidDidUrlFormat,
    #[error("didUrlPartLengthTooShort")]
    DidUrlPartLengthTooShort,
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
    #[error("Non-success server response")]
    NonSuccessResponse,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DidWebError {
    #[error("DID method not supported: {0}")]
    MethodNotSupported(String),
    #[error("Representation not supported: {0}")]
    RepresentationNotSupported(String),
    #[error("Invalid DID: {0}")]
    InvalidDid(String),
    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingErrorSource),
    #[error("URL parsing error: {0}")]
    HttpError(#[from] hyper::Error),
    #[error("Non-success server response: {0}")]
    NonSuccessResponse(StatusCode),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Error, Debug)]
pub enum ParsingErrorSource {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid encoding: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

impl From<serde_json::Error> for DidWebError {
    fn from(error: serde_json::Error) -> Self {
        DidWebError::ParsingError(ParsingErrorSource::JsonError(error))
    }
}

impl From<std::string::FromUtf8Error> for DidWebError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        DidWebError::ParsingError(ParsingErrorSource::Utf8Error(error))
    }
}
