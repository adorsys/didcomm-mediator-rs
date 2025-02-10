mod encryptor;
mod error;
mod repository;

pub use encryptor::KeyEncryption;
pub use error::{Error, ErrorKind};

use async_trait::async_trait;
use database::Repository;
use did_utils::jwk::Jwk;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use once_cell::sync::OnceCell;
use repository::SecretRepository;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;
use std::sync::Arc;
use thiserror::Error;
use tokio::{runtime::Handle, task::block_in_place};

static SECRETS_COLLECTION: OnceCell<Collection<Secrets>> = OnceCell::new();

/// A key store that manages cryptographic keys.
/// It is responsible for storing and retrieving cryptographic keys securely.
#[derive(Clone)]
pub struct Keystore<R: SecretRepository, E: KeyEncryption> {
    repository: Arc<R>,
    encryptor: Arc<E>,
}

impl<R: SecretRepository, E: KeyEncryption> Keystore<R, E> {
    /// Create a new key store with specified encyption backend.
    pub fn new(repository: R, encryptor: E) -> Self {
        Self {
            repository: Arc::new(repository),
            encryptor: Arc::new(encryptor),
        }
    }

    pub fn with_no_encryption() -> Self {
        Self::new(encryptor::NoEncryption)
    }

    /// Store a key in the keystore.
    pub async fn store<T: Serialize>(&self, kid: &str, key: &T) -> Result<(), Error> {
        let key_bytes = serde_json::to_vec(key)?;
        let encrypted_key = self.encryptor.encrypt(&key_bytes).await?;
        self.repository.store(kid, &encrypted_key).await?;
        Ok(())
    }

    /// Retrieve a key from the keystore with the specified key ID.
    pub async fn retrieve<T: for<'a>Deserialize<'a>>(&self, kid: &str) -> Result<Option<T>, Error> {
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

/// Represents a cryptographic secret
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Secrets {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub kid: String,

    pub secret_material: Vec<u8>,
}

// #[cfg(any(test, feature = "test-utils"))]
// pub mod tests {
//     use super::*;
//     use database::{Repository, RepositoryError};
//     use mongodb::bson::{doc, Bson, Document};
//     use serde_json::json;
//     use std::{collections::HashMap, sync::RwLock};

//     #[derive(Default)]
//     pub struct MockKeyStore {
//         secrets: RwLock<Vec<Secrets>>,
//     }

//     impl MockKeyStore {
//         pub fn new(secrets: Vec<Secrets>) -> Self {
//             Self {
//                 secrets: RwLock::new(secrets),
//             }
//         }

//         pub fn store(&self, secret: Secrets) {
//             self.secrets.write().unwrap().push(secret);
//         }
//     }

//     #[tokio::test]
//     async fn test_keystore_flow() {
//         let secret1: Jwk = serde_json::from_str(
//             r#"{
//                 "kty": "OKP",
//                 "crv": "X25519",
//                 "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ",
//                 "d": "0A8SSFkGHg3N9gmVDRnl63ih5fcwtEvnQu9912SVplY"
//             }"#,
//         )
//         .unwrap();

//         let secret2: Jwk = serde_json::from_str(
//             r#"{
//                 "kty": "OKP",
//                 "crv": "Ed25519",
//                 "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4",
//                 "d": "fI1u4riKKd99eox08GlThknq-vEJXcKBI28aiUqArLo"
//               }"#,
//         )
//         .unwrap();

//         let secrets = vec![
//             Secrets {
//                 id: Some(ObjectId::new()),
//                 kid: "1".to_string(),
//                 secret_material: secret1,
//             },
//             Secrets {
//                 id: Some(ObjectId::new()),
//                 kid: "2".to_string(),
//                 secret_material: secret2,
//             },
//         ];

//         let keystore = MockKeyStore::new(vec![]);

//         keystore.store(secrets[0].clone()).await.unwrap();
//         keystore.store(secrets[1].clone()).await.unwrap();

//         assert!(keystore
//             .find_one_by(doc! {"kid": "1"})
//             .await
//             .unwrap()
//             .is_some());
//         assert!(keystore
//             .find_one_by(doc! {"kid": "2"})
//             .await
//             .unwrap()
//             .is_some());

//         keystore.delete_one(secrets[0].id.unwrap()).await.unwrap();
//         assert!(keystore
//             .find_one_by(doc! {"kid": "1"})
//             .await
//             .unwrap()
//             .is_none());

//         assert_eq!(keystore.find_all().await.unwrap(), vec![secrets[1].clone()]);
//     }
// }
