//! A library for securely storing cryptographic keys.
//!
//! The library provides an abstraction for storing, retrieving, and deleting cryptographic keys.

#![warn(missing_docs)]

mod encryptor;
mod error;
mod repository;
#[cfg(any(test, feature = "test-utils"))]
mod tests;

// Public re-exports
pub use error::{Error, ErrorKind};
pub use repository::SecretRepository;
pub use encryptor::KeyEncryption;

use aws_sdk_kms::Client as KmsClient;
use encryptor::{aws_kms::AwsKmsEncryptor, plaintext::NoEncryption};
use repository::{mongodb::MongoSecretRepository, no_repo::NoRepository};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// The main type of the library.
///
/// The [`Keystore`] provides an abstraction for storing, retrieving, and deleting cryptographic keys.
/// It can support multiple storage backends and encryption mechanisms that can be configured.
///
/// # Usage
/// By default, the keystore is initialized with no encryption and no storage backend.
/// Storage and encryption backends can be specified using [`Keystore::with_repository`] and [`Keystore::with_encryptor`] methods.
///
/// # Example
/// ```no_run
/// use keystore::Keystore;
///
/// let repository = CustomRepository::new();
/// let encryptor = CustomEncryptor::new();
///
/// let keystore = Keystore::new()
///     .with_repository(repository)
///     .with_encryptor(encryptor);
///
/// let key = Jwk::generate_ed25519();
/// let key_id = "key1";
///
/// keystore.store(key_id, &key).await?;
///
/// let stored_key = keystore.retrieve(key_id).await?;
/// assert_eq!(stored_key, Some(key));
///
/// keystore.delete(key_id).await?;
/// ```
#[derive(Clone)]
pub struct Keystore {
    repository: Arc<dyn SecretRepository>,
    encryptor: Arc<dyn KeyEncryption>,
}

impl Keystore {
    /// Create a new key store instance.
    ///
    /// This function can be chained with:  
    /// * [`Keystore::with_repository`]  
    /// * [`Keystore::with_encryptor`]
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keystore::Keystore;
    ///
    /// let keystore = Keystore::new();
    /// ```
    pub fn new() -> Self {
        let repository = NoRepository;
        let encryptor = NoEncryption;
        Self {
            repository: Arc::new(repository),
            encryptor: Arc::new(encryptor),
        }
    }

    /// Create a new key store with mongoDB as storage backend.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keystore::Keystore;
    ///
    /// let keystore = Keystore::with_mongodb();
    /// ```
    pub fn with_mongodb() -> Self {
        let repository = MongoSecretRepository::new();
        let encryptor = NoEncryption;
        Self {
            repository: Arc::new(repository),
            encryptor: Arc::new(encryptor),
        }
    }

    /// Create a new key store with **AWS KMS** as encryption backend and mongoDB as storage backend.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keystore::Keystore;
    /// use aws_sdk_kms::Client;
    ///
    /// let config = aws_config::load_from_env().await;
    /// let client = Client::new(&config);
    /// let keystore = Keystore::with_aws_kms(client, "key_id".to_string());
    /// ```
    pub fn with_aws_kms(client: KmsClient, key_id: String) -> Self {
        let repository = MongoSecretRepository::new();
        let encryptor = AwsKmsEncryptor::new(client, key_id);
        Self {
            repository: Arc::new(repository),
            encryptor: Arc::new(encryptor),
        }
    }

    /// Set the repository backend for the key store.
    /// This method can be chained with [`Keystore::with_encryptor`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keystore::Keystore;
    ///
    /// let repository = CustomRepository::new();
    /// let keystore = Keystore::new().with_repository(repository);
    /// ```
    pub fn with_repository(self, repository: impl SecretRepository + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
            ..self
        }
    }

    /// Set the encryption backend for the key store.
    /// This method can be chained with [`Keystore::with_repository`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keystore::Keystore;
    ///
    /// let encryptor = CustomEncryptor::new();
    /// let keystore = Keystore::new().with_encryptor(encryptor);
    /// ```
    pub fn with_encryptor(self, encryptor: impl KeyEncryption + 'static) -> Self {
        Self {
            encryptor: Arc::new(encryptor),
            ..self
        }
    }

    /// Store a key in the keystore.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use keystore::Keystore;
    /// 
    /// let key = Jwk::generate_ed25519();
    /// let key_id = "key1";
    /// 
    /// keystore.store(key_id, &key).await?;
    /// ```
    pub async fn store<T: Serialize>(&self, kid: &str, key: &T) -> Result<(), Error> {
        let key_bytes = serde_json::to_vec(key)?;
        let encrypted_key = self.encryptor.encrypt(&key_bytes).await?;
        self.repository.store(kid, &encrypted_key).await?;
        Ok(())
    }

    /// Retrieve a key from the keystore with the specified key ID.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use keystore::Keystore;
    /// 
    /// let key: Option<Jwk> = keystore.retrieve("key1").await?;
    /// ```
    pub async fn retrieve<T: for<'a> Deserialize<'a>>(
        &self,
        kid: &str,
    ) -> Result<Option<T>, Error> {
        let secret = self.repository.find(kid).await?;

        if let Some(secret) = secret {
            let decrypted_key = self.encryptor.decrypt(&secret).await?;
            let key = serde_json::from_slice(&decrypted_key)?;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    /// Delete a key from the keystore with the specified key ID.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use keystore::Keystore;
    /// 
    /// keystore.delete("key1").await?;
    /// ```
    pub async fn delete(&self, kid: &str) -> Result<(), Error> {
        self.repository.delete(kid).await?;
        Ok(())
    }
}

impl Default for Keystore {
    fn default() -> Self {
        Self::new()
    }
}
