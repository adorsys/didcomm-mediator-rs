use async_trait::async_trait;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson, Document as BsonDocument},
    error::Error as MongoError,
    options::{ClientOptions, FindOptions},
    Client, Collection, Database,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// A trait that ensures the entity has an `id` field.
pub trait Identifiable {
    fn id(&self) -> Option<ObjectId>;
    fn set_id(&mut self, id: ObjectId);
}

/// Definition of custom errors for repository operations.
#[derive(Debug, Serialize, Deserialize, Error)]
pub enum RepositoryError {
    #[error("failed to convert to bson format")]
    BsonConversionError,
    #[error("generic: {0}")]
    Generic(String),
    #[error("missing identifier")]
    MissingIdentifier,
    #[error("target not found")]
    TargetNotFound,
}

static MONGO_DB: OnceCell<Arc<RwLock<Database>>> = OnceCell::new();

/// Get a handle to a database.
///
/// Many threads may call this function concurrently with different initializing functions,
/// but it is guaranteed that only one function will be executed.
pub fn get_or_init_database() -> Arc<RwLock<Database>> {
    MONGO_DB
        .get_or_init(|| {
            let mongo_uri = std::env::var("MONGO_URI").expect("MONGO_URI env variable required");
            let mongo_dbn = std::env::var("MONGO_DBN").expect("MONGO_DBN env variable required");

            // Create a handle to a database.
            let db = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move {
                    let client_options = ClientOptions::parse(mongo_uri)
                        .await
                        .expect("Failed to parse Mongo URI");
                    let client = Client::with_options(client_options)
                        .expect("Failed to create MongoDB client");

                    client.database(&mongo_dbn)
                })
            });

            Arc::new(RwLock::new(db))
        })
        .clone()
}

/// Definition of a trait for repository operations.
#[async_trait]
pub trait Repository<Entity>: Sync + Send
where
    Entity: Sized + Clone + Send + Sync + 'static,
    Entity: Identifiable + Unpin,
    Entity: Serialize + for<'de> Deserialize<'de>,
{
    fn get_collection(&self) -> Arc<RwLock<Collection<Entity>>>;

    async fn find_all(&self) -> Result<Vec<Entity>, RepositoryError> {
        let mut entities = Vec::new();
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let mut cursor = collection.read().await.find(None, None).await?;
        while cursor.advance().await? {
            entities.push(cursor.deserialize_current()?);
        }

        Ok(entities)
    }

    /// Counts all entities by filter.
    async fn count_by(&self, filter: BsonDocument) -> Result<usize, RepositoryError> {
        let collection = self.get_collection();
        // Lock the Mutex and get the Collection
        let collection = collection.read().await;
        Ok(collection
            .count_documents(filter, None)
            .await?
            .try_into()
            .map_err(|_| RepositoryError::Generic("count overflow".to_owned()))?)
    }

    async fn find_one(&self, id: ObjectId) -> Result<Option<Entity>, RepositoryError> {
        self.find_one_by(doc! {"_id": id}).await
    }

    async fn find_one_by(&self, filter: BsonDocument) -> Result<Option<Entity>, RepositoryError> {
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.read().await;
        Ok(collection.find_one(filter, None).await?)
    }

    /// Stores a new entity.
    async fn store(&self, mut entity: Entity) -> Result<Entity, RepositoryError> {
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.read().await;

        // Insert the new entity into the database
        let metadata = collection.insert_one(entity.clone(), None).await?;

        // Set the ID if it was inserted and return the updated entity
        if let Bson::ObjectId(oid) = metadata.inserted_id {
            entity.set_id(oid);
        }

        Ok(entity)
    }

    async fn find_all_by(
        &self,
        filter: BsonDocument,
        limit: Option<i64>,
    ) -> Result<Vec<Entity>, RepositoryError> {
        let find_options = FindOptions::builder().limit(limit).build();
        let mut entities = Vec::new();
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.read().await;

        // Retrieve all entities from the database
        let mut cursor = collection.find(filter, find_options).await?;
        while cursor.advance().await? {
            entities.push(cursor.deserialize_current()?);
        }

        Ok(entities)
    }

    async fn delete_one(&self, id: ObjectId) -> Result<(), RepositoryError> {
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.read().await;

        // Delete the entity from the database
        collection.delete_one(doc! {"_id": id}, None).await?;

        Ok(())
    }

    async fn update(&self, entity: Entity) -> Result<Entity, RepositoryError> {
        if entity.id().is_none() {
            return Err(RepositoryError::MissingIdentifier);
        }
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.read().await;

        // Update the entity in the database
        let metadata = collection
            .update_one(
                doc! {"_id": entity.id().unwrap()},
                doc! {"$set": bson::to_document(&entity).map_err(|_| RepositoryError::BsonConversionError)?},
                None,
            )
            .await?;

        if metadata.matched_count > 0 {
            Ok(entity)
        } else {
            Err(RepositoryError::TargetNotFound)
        }
    }
}

impl From<MongoError> for RepositoryError {
    fn from(error: MongoError) -> Self {
        RepositoryError::Generic(error.to_string())
    }
}
