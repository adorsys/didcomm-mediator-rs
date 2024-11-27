#![allow(unused_imports)]

use async_trait::async_trait;
use cocoon::MiniCocoon;
use database::{Identifiable, Repository, RepositoryError};
use did_utils::jwk::Jwk;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    options::CountOptions,
    Collection,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::{borrow::Borrow, sync::Arc};
use tokio::{runtime::EnterGuard, sync::RwLock};

static SECRETS_COLLECTION: OnceCell<Collection<Secrets>> = OnceCell::new();
#[async_trait]
pub trait Material {
    async fn securestore(&self, secret: Secrets) -> Result<Secrets, RepositoryError>;
    async fn find_one_by(&self, kid: String) -> Result<Option<Secrets>, RepositoryError>;
}

#[async_trait]
impl Material for SecretStore {
    async fn securestore(&self, secret: Secrets) -> Result<Secrets, RepositoryError> {
        // read master key for encryption
        let master_key = std::env::var("MASTER_KEY").unwrap_or_default();

        let mut secret = secret;

        let seed = &[0; 32];
        let secret_material = secret.secret_material.as_bytes();
        let mut cocoon = MiniCocoon::from_key(master_key.as_bytes(), seed);
        let wrapped_key = cocoon.wrap(secret_material).unwrap_or_default();
        secret.secret_material = String::from_utf8(wrapped_key).unwrap_or_default();

        // Insert the new entity into the database
        let metadata = self
            .keystore
            .collection
            .insert_one(secret.clone(), None)
            .await?;

        // Set the ID if it was inserted and return the updated entity
        if let Bson::ObjectId(oid) = metadata.inserted_id {
            secret.set_id(oid);
        }

        Ok(secret)
    }

    async fn find_one_by(&self, kid: String) -> Result<Option<Secrets>, RepositoryError> {
        let collection = self.keystore.clone();

        let secret = collection
            .collection
            .find_one(doc! {"kid": kid}, None)
            .await?;
        if let Some(mut secrets) = secret {
            let wrapped_secret_material = secrets.secret_material;
            let master_key = std::env::var("MASTER_KEY").unwrap_or_default();
            let seed = &[0; 32];
            let cocoon = MiniCocoon::from_key(master_key.as_bytes(), seed);
            let unwrap_secret = cocoon
                .unwrap(wrapped_secret_material.as_bytes())
                .unwrap_or_default();
            secrets.secret_material = String::from_utf8(unwrap_secret).unwrap_or_default();
            Ok(Some(secrets))
        } else {
            Ok(None)
        }
    }
}

pub struct SecretStore {
    keystore: KeyStore<Secrets>,
}

impl SecretStore {
    pub fn new() -> Self {
        let keystore = KeyStore::get();
        Self { keystore }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Secrets {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub kid: String,

    pub secret_material: String,
}

impl Identifiable for Secrets {
    fn id(&self) -> Option<ObjectId> {
        self.id
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}

#[derive(Debug, Clone)]
pub struct KeyStore<T>
where
    T: Sized + Clone + Send + Sync + 'static,
    T: Identifiable + Unpin,
    T: Serialize + for<'de> Deserialize<'de>,
{
    collection: Collection<T>,
}

impl KeyStore<Secrets> {
    /// Create a new keystore with default Secrets type.
    ///
    /// Calling this method many times will return the same keystore instance.
    pub fn new() -> KeyStore<Secrets> {
        let collection = SECRETS_COLLECTION
            .get_or_init(|| {
                let db = database::get_or_init_database();
                let task = async move {
                    let db_lock = db.write().await;
                    db_lock.collection::<Secrets>("secrets").clone()
                };
                let collection = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(task)
                });
                collection
            })
            .clone();

        KeyStore { collection }
    }

    /// Retrieve the keystore instance.
    ///
    /// If there is no keystore instance, a new one will be created only once.
    pub fn get() -> KeyStore<Secrets> {
        Self::new()
    }
}

impl<T> KeyStore<T>
where
    T: Sized + Clone + Send + Sync + 'static,
    T: Identifiable + Unpin,
    T: Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new keystore with specified type
    pub fn new_generic() -> Self {
        let db = database::get_or_init_database();
        let collection = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let db_lock = db.write().await;
                db_lock.collection("secrets").clone()
            });

        Self { collection }
    }
}

#[async_trait]
impl Repository<Secrets> for SecretStore {
    fn get_collection(&self) -> Arc<RwLock<Collection<Secrets>>> {
        Arc::new(RwLock::new(self.keystore.collection.clone()))
    }

    async fn store(&self, entity: Secrets) -> Result<Secrets, RepositoryError> {
        let secrets = self.securestore(entity).await?;
        Ok(secrets)
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use super::*;
    use database::{Repository, RepositoryError};
    use mongodb::bson::{doc, Bson, Document};
    use rand::Rng;
    use serde_json::json;
    use std::{borrow::Borrow, collections::HashMap, sync::RwLock};

    #[derive(Default)]
    pub struct MockKeyStore {
        secrets: RwLock<Vec<Secrets>>,
    }
    #[derive(Default)]
    pub struct MockSecretStore {
        keystore: RwLock<Vec<Secrets>>,
    }

    impl MockSecretStore {
        pub fn new(secrets: Vec<Secrets>) -> Self {
            Self {
                keystore: RwLock::new(secrets),
            }
        }
    }
    impl MockKeyStore {
        pub fn new(secrets: Vec<Secrets>) -> Self {
            Self {
                secrets: RwLock::new(secrets),
            }
        }
    }

    #[async_trait]
    impl Repository<Secrets> for MockKeyStore {
        // Implement a dummy get_collection method
        fn get_collection(&self) -> Arc<tokio::sync::RwLock<Collection<Secrets>>> {
            // In-memory, we don't have an actual collection, but we can create a dummy Arc<Mutex> for compatibility.
            unimplemented!("This is a mock repository, no real collection exists.")
        }

        async fn find_all(&self) -> Result<Vec<Secrets>, RepositoryError> {
            Ok(self.secrets.read().unwrap().clone())
        }

        async fn find_one(&self, secrets_id: ObjectId) -> Result<Option<Secrets>, RepositoryError> {
            self.find_one_by(doc! {"_id": secrets_id}).await
        }

        async fn find_one_by(&self, filter: Document) -> Result<Option<Secrets>, RepositoryError> {
            let filter: HashMap<String, Bson> = filter.into_iter().collect();
            Ok(self
                .secrets
                .read()
                .unwrap()
                .iter()
                .find(|s| {
                    if let Some(kid) = filter.get("kid") {
                        if json!(s.kid) != json!(kid) {
                            return false;
                        }
                    }
                    true
                })
                .cloned())
        }

        async fn store(&self, secrets: Secrets) -> Result<Secrets, RepositoryError> {
            self.secrets.write().unwrap().push(secrets.clone());
            Ok(secrets)
        }

        async fn update(&self, secrets: Secrets) -> Result<Secrets, RepositoryError> {
            let mut secrets_list = self.secrets.write().unwrap();
            if let Some(pos) = secrets_list.iter().position(|s| s.id == secrets.id) {
                secrets_list[pos] = secrets.clone();
                Ok(secrets)
            } else {
                Err(RepositoryError::TargetNotFound)
            }
        }

        async fn delete_one(&self, secrets_id: ObjectId) -> Result<(), RepositoryError> {
            let mut secrets_list = self.secrets.write().unwrap();
            if let Some(pos) = secrets_list.iter().position(|s| s.id == Some(secrets_id)) {
                secrets_list.remove(pos);
            }
            Ok(())
        }
    }

    #[async_trait]
    impl Material for MockSecretStore {
        async fn securestore(&self, secrets: Secrets) -> Result<Secrets, RepositoryError> {
            // read master key for encryption
        //    let master_key = "masterkey".to_string();
            let master_key = rand::thread_rng().gen::<[u8; 32]>();

            let mut secret = secrets;

            let seed = &[0; 32];
            let secret_material = secret.secret_material.as_bytes();
            let mut cocoon = MiniCocoon::from_key(&master_key, seed);
            let wrapped_key = cocoon.wrap(secret_material).unwrap();
            secret.secret_material = String::from_utf8(wrapped_key.clone()).unwrap();
           let a= String::from_utf8(wrapped_key).unwrap();

            // Insert the new entity into the database
            self.keystore.write().unwrap().push(secret.clone());
            Ok(secret)
        }

        async fn find_one_by(&self, kid: String) -> Result<Option<Secrets>, RepositoryError> {
            let secret = self
                .keystore
                .read()
                .unwrap()
                .iter()
                .find(|s| {
                    if json!(s.kid) != json!(kid) {
                        return false;
                    }
                    true
                })
                .cloned();

            if let Some(mut secrets) = secret {
                let wrapped_secret_material = secrets.secret_material;
                let master_key = "masterkey".to_string();
                let seed = &[0; 32];
                let cocoon = MiniCocoon::from_key(master_key.as_bytes(), seed);
                let unwrap_secret = cocoon
                    .unwrap(wrapped_secret_material.as_bytes())
                    .unwrap_or_default();
                secrets.secret_material = String::from_utf8(unwrap_secret).unwrap_or_default();
                Ok(Some(secrets))
            } else {
                Ok(None)
            }
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
        let secret1 = serde_json::to_vec(&secret1).unwrap();

        let secret1 = String::from_utf8(secret1).unwrap_or_default();

        let secret2 = serde_json::to_vec(&secret2).unwrap();

        let secret2 = String::from_utf8(secret2).unwrap_or_default();

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
        let secretstore = MockSecretStore::new(vec![]);
        secretstore.securestore(secrets[0].clone()).await.unwrap();
        secretstore.securestore(secrets[1].clone()).await.unwrap();
        

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
