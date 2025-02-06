mod backend;

use async_trait::async_trait;
use bson::doc;
use database::{Identifiable, Repository, RepositoryError};
use did_utils::jwk::Jwk;
use mongodb::{bson::oid::ObjectId, Collection};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;
use std::sync::Arc;
use thiserror::Error;
use tokio::{runtime::Handle, task::block_in_place};

static SECRETS_COLLECTION: OnceCell<Collection<Secrets>> = OnceCell::new();

/// Errors that can occur during encryption operations.
#[derive(Debug, Error)]
pub enum KeyEncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailure(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailure(String),
}

/// Errors that can occur during key store operations.
#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("Repository error: {0}")]
    RepositoryFailure(#[from] RepositoryError),
    #[error("Encryption error: {0}")]
    KeyEncryptionFailure(#[from] KeyEncryptionError),
    #[error("Serialization error: {0}")]
    SerializationFailure(#[from] SerdeError),
}

/// Abstract interface for key encryption backends.
#[async_trait]
pub trait KeyEncryption: Send + Sync {
    /// Encrypt plaintext key material.
    async fn encrypt(&self, key_material: &[u8]) -> Result<Vec<u8>, KeyEncryptionError>;

    /// Decrypt encrypted key material.
    async fn decrypt(&self, encrypted_key: &[u8]) -> Result<Vec<u8>, KeyEncryptionError>;
}

/// A key store that manages cryptographic keys.
/// It is responsible for storing and retrieving cryptographic keys securely.
#[derive(Clone)]
pub struct Keystore {
    repository: Collection<Secrets>,
    encryptor: Arc<dyn KeyEncryption>,
}

impl Keystore {
    /// Create a new key store with specified encyption backend.
    pub fn new<T: KeyEncryption + 'static>(encryptor: T) -> Self {
        let repository = SECRETS_COLLECTION
            .get_or_init(|| {
                let db = database::get_or_init_database();
                let task = async move {
                    let db_lock = db.write().await;
                    db_lock.collection::<Secrets>("secrets").clone()
                };
                block_in_place(|| Handle::current().block_on(task))
            })
            .clone();

        Self {
            repository,
            encryptor: Arc::new(encryptor),
        }
    }

    pub fn with_no_encryption() -> Self {
        Self::new(backend::NoEncryption)
    }

    /// Store a key in the keystore.
    pub async fn store(&self, kid: String, key: Jwk) -> Result<(), KeyStoreError> {
        let key_bytes = serde_json::to_vec(&key)
            .map_err(|err| KeyStoreError::SerializationFailure(err.into()))?;
        let encrypted_key = self.encryptor.encrypt(&key_bytes).await?;
        let secret = Secrets {
            id: None,
            kid,
            secret_material: encrypted_key,
        };
        self.repository
            .insert_one(secret)
            .await
            .map_err(|err| KeyStoreError::RepositoryFailure(err.into()))?;
        Ok(())
    }

    /// Retrieve a key from the keystore with the specified key ID.
    pub async fn get(&self, kid: &str) -> Result<Option<Vec<u8>>, KeyStoreError> {
        let filter = doc! {"kid": kid};
        let projection = doc! {"_id": 0, "secret_material": 1};

        let secret = self
            .repository
            .find_one(filter)
            .projection(projection)
            .await
            .map_err(|err| KeyStoreError::RepositoryFailure(err.into()))?;

        if let Some(secret) = secret {
            let key = self.encryptor.decrypt(&secret.secret_material).await?;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }
}

/// Represents a cryptographic secret
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Secrets {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub kid: String,

    pub secret_material: Vec<u8>,
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use super::*;
    use database::{Repository, RepositoryError};
    use mongodb::bson::{doc, Bson, Document};
    use serde_json::json;
    use std::{collections::HashMap, sync::RwLock};

    #[derive(Default)]
    pub struct MockKeyStore {
        secrets: RwLock<Vec<Secrets>>,
    }

    impl MockKeyStore {
        pub fn new(secrets: Vec<Secrets>) -> Self {
            Self {
                secrets: RwLock::new(secrets),
            }
        }

        pub fn store(&self, secret: Secrets) {
            self.secrets.write().unwrap().push(secret);
        }
    }

    #[tokio::test]
    async fn test_keystore_flow() {
        let secret1: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ",
                "d": "0A8SSFkGHg3N9gmVDRnl63ih5fcwtEvnQu9912SVplY"
            }"#,
        )
        .unwrap();

        let secret2: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4",
                "d": "fI1u4riKKd99eox08GlThknq-vEJXcKBI28aiUqArLo"
              }"#,
        )
        .unwrap();

        let secrets = vec![
            Secrets {
                id: Some(ObjectId::new()),
                kid: "1".to_string(),
                secret_material: secret1,
            },
            Secrets {
                id: Some(ObjectId::new()),
                kid: "2".to_string(),
                secret_material: secret2,
            },
        ];

        let keystore = MockKeyStore::new(vec![]);

        keystore.store(secrets[0].clone()).await.unwrap();
        keystore.store(secrets[1].clone()).await.unwrap();

        assert!(keystore
            .find_one_by(doc! {"kid": "1"})
            .await
            .unwrap()
            .is_some());
        assert!(keystore
            .find_one_by(doc! {"kid": "2"})
            .await
            .unwrap()
            .is_some());

        keystore.delete_one(secrets[0].id.unwrap()).await.unwrap();
        assert!(keystore
            .find_one_by(doc! {"kid": "1"})
            .await
            .unwrap()
            .is_none());

        assert_eq!(keystore.find_all().await.unwrap(), vec![secrets[1].clone()]);
    }
}
