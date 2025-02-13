#![warn(missing_docs)]

//! A library for managing cryptographic keys.

mod encryptor;
mod error;
mod repository;
#[cfg(any(test, feature = "test-utils"))]
mod tests;

pub use error::{Error, ErrorKind};

use aws_sdk_kms::Client as KmsClient;
use encryptor::{aws_kms::AwsKmsEncryptor, plaintext::NoEncryption, KeyEncryption};
use repository::{mongodb::MongoSecretRepository, no_repo::NoRepository, SecretRepository};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
#[cfg(any(test, feature = "test-utils"))]
use tests::MockSecretRepository;

/// The main type of the library.
///
/// The `Keystore` provides an abstraction for storing, retrieving, and deleting cryptographic keys.
/// It can support multiple storage backends and encryption mechanisms that can be configured.
///
/// # Usage
/// By default, the keystore is initialized with no encryption and no storage backend.
/// Storage and encryption backends can be specified using [`Keystore::with_repository`] and [`Keystore::with_encryptor`] methods.
///
/// # Example
/// ```no_run
/// use keystore::{Keystore, NoEncryption};
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
    /// let repository = CustomRepository::new();
    /// let encryptor = CustomEncryptor::new();
    ///
    /// let keystore = Keystore::new()
    ///     .with_repository(repository)
    ///     .with_encryptor(encryptor);
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
    /// use keystore::{Keystore, NoEncryption};
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

    /// Create a new key store with mocked repository and encryption backends.
    /// This will be use for testing purposes.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn with_mock_configs<T>(secrets: Vec<(String, T)>) -> Self
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        let mock_repository = MockSecretRepository::new(secrets);
        let encryptor = NoEncryption;
        Self {
            repository: Arc::new(mock_repository),
            encryptor: Arc::new(encryptor),
        }
    }

    /// Store a key in the keystore.
    pub async fn store<T: Serialize>(&self, kid: &str, key: &T) -> Result<(), Error> {
        let key_bytes = serde_json::to_vec(key)?;
        let encrypted_key = self.encryptor.encrypt(&key_bytes).await?;
        self.repository.store(kid, &encrypted_key).await?;
        Ok(())
    }

    /// Retrieve a key from the keystore with the specified key ID.
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
