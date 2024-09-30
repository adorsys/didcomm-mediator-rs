use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("File error: {0}")]
    FileError(std::io::Error),
    #[error("JwkConversionError")] 
    JwkConversionError,
    #[error("KeyPairGenerationError")]
    KeyPairGenerationError,
    #[error("non compliant")]
    NonCompliant,
    #[error("not found")]
    NotFound,
    #[error("parse error")]
    ParseError(serde_json::Error),
    #[error("serde error")]
    SerdeError(serde_json::Error),
    #[error("Encryption error: {0}")]
    EncryptionError(chacha20poly1305::Error),
    #[error("Decryption error: {0}")]
    DecryptionError(chacha20poly1305::Error),
}
impl From<std::io::Error> for KeystoreError {
    fn from(err: std::io::Error) -> Self {
        KeystoreError::FileError(err)
    }
}