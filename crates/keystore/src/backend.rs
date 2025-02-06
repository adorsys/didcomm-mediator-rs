pub(crate) mod aws_kms;

use crate::{KeyEncryption, KeyEncryptionError};
use async_trait::async_trait;

pub struct NoEncryption;

#[async_trait]
impl KeyEncryption for NoEncryption {
    async fn encrypt(&self, key_material: &[u8]) -> Result<Vec<u8>, KeyEncryptionError> {
        Ok(key_material.to_vec())
    }

    async fn decrypt(&self, encrypted_key: &[u8]) -> Result<Vec<u8>, KeyEncryptionError> {
        Ok(encrypted_key.to_vec())
    }
}
