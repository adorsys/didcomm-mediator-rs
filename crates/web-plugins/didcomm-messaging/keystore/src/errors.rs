use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("File operation failed: {0}")]
    FileError(#[from] std::io::Error),

    #[error("JWK conversion failed")]
    JwkConversionError,

    #[error("Key pair generation failed")]
    KeyPairGenerationError,

    #[error("Non-compliant data")]
    NonCompliant,

    #[error("Item not found")]
    NotFound,

    #[error("Failed to parse JSON data: {0}")]
    ParseError(serde_json::Error),

    #[error("Serialization error: {0}")]
    SerializationError(serde_json::Error),

    #[error("Deserialization error: {0}")]
    DeserializationError(serde_json::Error),

    #[error("Encryption failed: {0}")]
    EncryptionError(chacha20poly1305::Error),

    #[error("Decryption failed: {0}")]
    DecryptionError(chacha20poly1305::Error),
}
