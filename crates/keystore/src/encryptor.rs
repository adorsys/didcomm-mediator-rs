pub(crate) mod aws_kms;
pub(crate) mod plaintext;

use async_trait::async_trait;

use crate::Error;

/// Abstract interface for key encryption backends.
#[async_trait]
pub trait KeyEncryption: Send + Sync {
    /// Encrypt plaintext key material.
    async fn encrypt(&self, key_material: &[u8]) -> Result<Vec<u8>, Error>;

    /// Decrypt encrypted key material.
    async fn decrypt(&self, encrypted_key: &[u8]) -> Result<Vec<u8>, Error>;
}
