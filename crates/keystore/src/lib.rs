use async_trait::async_trait;
use database::{Identifiable, Repository};
use did_utils::jwk::Jwk;
use mongodb::{bson::oid::ObjectId, Collection};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

static SECRETS_COLLECTION: OnceCell<Collection<Secrets>> = OnceCell::new();

/// Represents a cryptographic secret stored in the keystore.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Secrets {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub kid: String,

    pub secret_material: Jwk,
}

impl Identifiable for Secrets {
    fn id(&self) -> Option<ObjectId> {
        self.id
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}

/// A keystore for managing cryptographic secrets.
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
impl<T> Repository<T> for KeyStore<T>
where
    T: Sized + Clone + Send + Sync + 'static,
    T: Identifiable + Unpin,
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn get_collection(&self) -> Arc<RwLock<Collection<T>>> {
        Arc::new(RwLock::new(self.collection.clone()))
    }
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
