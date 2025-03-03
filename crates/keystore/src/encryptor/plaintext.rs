use async_trait::async_trait;

use crate::{Error, KeyEncryption};

pub(crate) struct NoEncryption;

#[async_trait]
impl KeyEncryption for NoEncryption {
    async fn encrypt(&self, key_material: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(key_material.to_vec())
    }

    async fn decrypt(&self, encrypted_key: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(encrypted_key.to_vec())
    }
}
