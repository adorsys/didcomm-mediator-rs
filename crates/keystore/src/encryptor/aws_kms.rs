use async_trait::async_trait;
use aws_sdk_kms::{primitives::Blob, Client};

use crate::{Error, ErrorKind, KeyEncryption};

/// Amazon Web Services KMS encryption backend.
pub(crate) struct AwsKmsEncryptor {
    client: Client,
    key_id: String,
}

impl AwsKmsEncryptor {
    /// Create a new AWS KMS encryption backend.
    pub(crate) fn new(client: Client, key_id: String) -> Self {
        Self { client, key_id }
    }
}

#[async_trait]
impl KeyEncryption for AwsKmsEncryptor {
    async fn encrypt(&self, key_material: &[u8]) -> Result<Vec<u8>, Error> {
        let plaintext = Blob::new(key_material);
        let resp = self
            .client
            .encrypt()
            .key_id(&self.key_id)
            .plaintext(plaintext)
            .send()
            .await?;
        let ciphertext = resp
            .ciphertext_blob
            .ok_or_else(|| Error::msg(ErrorKind::EncryptionFailure, "Missing ciphertext"))?;

        Ok(ciphertext.as_ref().to_vec())
    }

    async fn decrypt(&self, encrypted_key: &[u8]) -> Result<Vec<u8>, Error> {
        let ciphertext = Blob::new(encrypted_key);
        let resp = self
            .client
            .decrypt()
            .key_id(&self.key_id)
            .ciphertext_blob(ciphertext)
            .send()
            .await?;
        let plaintext = resp
            .plaintext
            .ok_or_else(|| Error::msg(ErrorKind::DecryptionFailure, "Missing plaintext"))?;

        Ok(plaintext.as_ref().to_vec())
    }
}
