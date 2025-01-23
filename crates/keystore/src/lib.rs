use async_trait::async_trait;
use cocoon::MiniCocoon;
use database::{Identifiable, Repository, RepositoryError};
use did_utils::jwk::Jwk;
use mongodb::{
    bson::{oid::ObjectId, to_vec, Bson, Document as BsonDocument},
    Collection,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

static SECRETS_COLLECTION: OnceCell<Collection<WrapSecret>> = OnceCell::new();

/// Definition of a trait for secure repository operations.
#[async_trait]
pub trait SecureRepository<Entity>: Sync + Send
where
    Entity: Sized + Send + Sync + 'static + Clone,
    Entity:
        Identifiable + Unpin + Into<Secrets> + From<WrapSecret> + Into<WrapSecret> + From<Secrets>,
    Entity: Serialize + for<'de> Deserialize<'de>,
{
    fn get_collection(&self) -> Arc<RwLock<Collection<Entity>>>;
    // does not handle errors since they are all fatal
    async fn secure_store(
        &self,
        entity: Entity,
        master_key: [u8; 32],
    ) -> Result<Entity, RepositoryError> {
        // convert and entity into a Wrapsecret
        let secret: WrapSecret = entity.into();

        // hardecoded seed to always correspond to master key
        let seed = [0; 32];

        let mut cocoon = MiniCocoon::from_key(&master_key, &seed);
        let wrapped_jwk = cocoon.wrap(&secret.secret_material).unwrap();
        let wrapped_secret = WrapSecret {
            id: None,
            kid: secret.kid.clone(),
            secret_material: wrapped_jwk,
        };
        // convert wrapped secret into entity
        let secret_entity: Entity = wrapped_secret.into();

        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.read().await;

        // Insert the new entity into the database
        collection.insert_one(secret_entity.clone()).await?;

        Ok(secret_entity)
    }
    async fn find_key_by(
        &self,
        filter: BsonDocument,
        master_key: [u8; 32],
    ) -> Result<Option<Secrets>, RepositoryError> {
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.read().await;
        let entity = collection.find_one(filter).await?;
        if let Some(wrapsecret) = entity {
            // hardecoded seed to always correspond to master key
            let seed = [0; 32];
            let wrapped_secret: WrapSecret = wrapsecret.into();
            let cocoon = MiniCocoon::from_key(&master_key, &seed);

            let unwrap_secret_material = cocoon
                .unwrap(&wrapped_secret.secret_material)
                .map_err(|_| RepositoryError::DecryptionError)?;

            let secret_material: BsonDocument = bson::from_slice(&unwrap_secret_material).unwrap();
            let secret_material = serde_json::to_value(secret_material).unwrap();

            let jwk: Jwk = serde_json::from_value(secret_material)
                .map_err(|_| RepositoryError::JwkDeserializationError)?;

            let unwrap_secret = Secrets {
                id: None,
                kid: wrapped_secret.kid,
                secret_material: jwk,
            };
            // convert secret to entity
            // let unwrap_entity: Entity = unwrap_secret.into();
            Ok(Some(unwrap_secret))
        } else {
            Ok(None)
        }
    }
}
#[async_trait]
impl<T> SecureRepository<T> for KeyStore<T>
where
    T: Sized + Clone + Send + Sync + 'static,
    T: Identifiable + Unpin + From<Secrets>,
    T: From<WrapSecret> + Into<WrapSecret> + Into<Secrets>,
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn get_collection(&self) -> Arc<RwLock<Collection<T>>> {
        Arc::new(RwLock::new(self.collection.clone()))
    }
}

impl From<Secrets> for WrapSecret {
    fn from(value: Secrets) -> Self {
        let secret_material = to_vec(&value.secret_material).unwrap_or_default();
        WrapSecret {
            id: value.id,
            kid: value.kid,
            secret_material,
        }
    }
}
impl From<WrapSecret> for Secrets {
    fn from(value: WrapSecret) -> Self {
        let secret_material = value.secret_material;
        let secret_material: BsonDocument = bson::from_slice(&secret_material).unwrap();
        let secret_material = serde_json::to_value(secret_material).unwrap();
        let secret_material: Jwk = serde_json::from_value(secret_material).unwrap();
        Secrets {
            id: value.id,
            kid: value.kid,
            secret_material,
        }
    }
}

/// Represents a cryptographic secret stored in the keystore.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WrapSecret {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub kid: String,

    // wrap secrets as Vec<u8>
    pub secret_material: Vec<u8>,
}
impl Identifiable for WrapSecret {
    fn id(&self) -> Option<ObjectId> {
        self.id
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}

/// Represents a cryptographic secret
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

impl Default for KeyStore<WrapSecret> {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyStore<WrapSecret> {
    /// Create a new keystore with default Secrets type.
    ///
    /// Calling this method many times will return the same keystore instance.
    pub fn new() -> KeyStore<WrapSecret> {
        let collection = SECRETS_COLLECTION
            .get_or_init(|| {
                let db = database::get_or_init_database();
                let task = async move {
                    let db_lock = db.write().await;
                    db_lock.collection::<WrapSecret>("secrets").clone()
                };
                tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(task))
            })
            .clone();

        KeyStore { collection }
    }

    /// Retrieve the keystore instance.
    ///
    /// If there is no keystore instance, a new one will be created only once.
    pub fn get() -> KeyStore<WrapSecret> {
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
    impl SecureRepository<Secrets> for MockKeyStore {
        fn get_collection(&self) -> Arc<tokio::sync::RwLock<Collection<Secrets>>> {
            // In-memory, we don't have an actual collection, but we can create a dummy Arc<Mutex> for compatibility.
            unimplemented!("This is a mock repository, no real collection exists.")
        }
    }
    #[async_trait]
    impl SecureRepository<WrapSecret> for MockKeyStore {
        fn get_collection(&self) -> Arc<tokio::sync::RwLock<Collection<WrapSecret>>> {
            // In-memory, we don't have an actual collection, but we can create a dummy Arc<Mutex> for compatibility.
            unimplemented!("This is a mock repository, no real collection exists.")
        }

        async fn find_key_by(
            &self,
            filter: Document,
            _master_key: [u8; 32],
        ) -> Result<Option<Secrets>, RepositoryError> {
            let filter: HashMap<String, Bson> = filter.into_iter().collect();
            let secret = self
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
                .cloned();
            // let secret: WrapSecret = secret.unwrap().into();
            Ok(secret)
        }
        async fn secure_store(
            &self,
            secrets: WrapSecret,
            _master_key: [u8; 32],
        ) -> Result<WrapSecret, RepositoryError> {
            self.secrets.write().unwrap().push(secrets.clone().into());
            Ok(secrets)
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
