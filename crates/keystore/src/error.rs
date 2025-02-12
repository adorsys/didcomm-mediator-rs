use core::fmt::{Debug, Display};
use std::error::Error as StdError;

use aws_sdk_kms::operation::{decrypt::DecryptError, encrypt::EncryptError};
use serde_json::error::Category;

/// Kind of error that can occur during key store operations.
#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorKind {
    /// The error was caused by operations on the database.
    #[error("Repository error")]
    RepositoryFailure,
    /// The error occurred when trying to encrypt the key.
    #[error("Encryption failure")]
    EncryptionFailure,
    /// The error occurred when trying to decrypt the key.
    #[error("Decryption failure")]
    DecryptionFailure,
    /// The error was caused by a failure to serialize or deserialize the key.
    #[error("The key is malformed")]
    MalformedKey,
    /// Another error that is not categorized occurred.
    #[error("Other error")]
    Other,
}

/// Represents all possible errors that can occur during key store operations.
pub struct Error {
    kind: ErrorKind,
    source: eyre::Report,
}

impl Error {
    /// Returns the kind of the error that occurred.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the lowest level error that caused this error.
    pub fn source(&self) -> &(dyn StdError + 'static) {
        self.source.root_cause()
    }

    /// Returns the context of the error.
    pub fn context(&self) -> &(dyn StdError) {
        self.source.as_ref()
    }

    pub(crate) fn new<E>(kind: ErrorKind, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Error {
            kind,
            source: eyre::Report::new(source),
        }
    }

    pub(crate) fn msg<M>(kind: ErrorKind, msg: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error {
            kind,
            source: eyre::Report::msg(msg),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("context", &self.context())
            .field("source", &self.source())
            .finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.context())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        match err.classify() {
            Category::Io | Category::Eof => Error::new(ErrorKind::Other, err),
            _ => Error::new(ErrorKind::MalformedKey, err),
        }
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(err: mongodb::error::Error) -> Self {
        Error::new(ErrorKind::RepositoryFailure, err)
    }
}

impl From<aws_sdk_kms::error::SdkError<EncryptError>> for Error {
    fn from(err: aws_sdk_kms::error::SdkError<EncryptError>) -> Self {
        Error::new(ErrorKind::EncryptionFailure, err)
    }
}

impl From<aws_sdk_kms::error::SdkError<DecryptError>> for Error {
    fn from(err: aws_sdk_kms::error::SdkError<DecryptError>) -> Self {
        Error::new(ErrorKind::DecryptionFailure, err)
    }
}
