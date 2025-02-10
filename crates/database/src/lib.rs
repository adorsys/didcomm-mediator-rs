use async_trait::async_trait;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson, Document as BsonDocument},
    error::Error as MongoError,
    options::{ClientOptions, FindOptions},
    Client, Collection, Database,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    #[error("decryption error")]
    DecryptionError,
    #[error("error deserializing to jwk")]
    JwkDeserializationError,
}

static MONGO_DB: OnceCell<Database> = OnceCell::new();

/// Get a handle to a database.
///
/// Many threads may call this function concurrently with different initializing functions,
/// but it is guaranteed that only one function will be executed.
pub fn get_or_init_database() -> Database {
    MONGO_DB
        .get_or_init(|| {
            let mongo_uri = std::env::var("MONGO_URI").expect("MONGO_URI env variable required");
            let mongo_dbn = std::env::var("MONGO_DBN").expect("MONGO_DBN env variable required");

            // Create a handle to a database.
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move {
                    let client_options = ClientOptions::parse(mongo_uri)
                        .await
                        .expect("Failed to parse Mongo URI");
                    let client = Client::with_options(client_options)
                        .expect("Failed to create MongoDB client");

                    client.database(&mongo_dbn)
                })
            })
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
    /// Get a handle to a collection.
    fn get_collection(&self) -> Collection<Entity>;

    /// Retrieve all entities from the database.
    async fn find_all(&self) -> Result<Vec<Entity>, RepositoryError> {
        let mut entities = Vec::new();
        let collection = self.get_collection();

        let mut cursor = collection.find(doc! {}).await?;
        while cursor.advance().await? {
            entities.push(cursor.deserialize_current()?);
        }

        Ok(entities)
    }

    /// Gets the number of documents matching `filter`.
    async fn count_by(&self, filter: BsonDocument) -> Result<usize, RepositoryError> {
        let collection = self.get_collection();
        Ok(collection
            .count_documents(filter)
            .await?
            .try_into()
            .map_err(|_| RepositoryError::Generic("count overflow".to_owned()))?)
    }

    /// Find an entity by `id`.
    async fn find_one(&self, id: ObjectId) -> Result<Option<Entity>, RepositoryError> {
        self.find_one_by(doc! {"_id": id}).await
    }

    /// Find an entity matching `filter`.
    async fn find_one_by(&self, filter: BsonDocument) -> Result<Option<Entity>, RepositoryError> {
        let collection = self.get_collection();

        Ok(collection.find_one(filter).await?)
    }

    /// Stores a new entity.
    async fn store(&self, mut entity: Entity) -> Result<Entity, RepositoryError> {
        let collection = self.get_collection();

        // Insert the new entity into the database
        let metadata = collection.insert_one(entity.clone()).await?;

        // Set the ID if it was inserted and return the updated entity
        if let Bson::ObjectId(oid) = metadata.inserted_id {
            entity.set_id(oid);
        }

        Ok(entity)
    }

    /// Find all entities matching `filter`.
    /// If `limit` is set, only the first `limit` entities are returned.
    async fn find_all_by(
        &self,
        filter: BsonDocument,
        limit: Option<i64>,
    ) -> Result<Vec<Entity>, RepositoryError> {
        let find_options = FindOptions::builder().limit(limit).build();
        let mut entities = Vec::new();
        let collection = self.get_collection();

        // Retrieve all entities from the database
        let mut cursor = collection.find(filter).with_options(find_options).await?;
        while cursor.advance().await? {
            entities.push(cursor.deserialize_current()?);
        }

        Ok(entities)
    }

    /// Deletes an entity by `id`.
    async fn delete_one(&self, id: ObjectId) -> Result<(), RepositoryError> {
        let collection = self.get_collection();

        // Delete the entity from the database
        collection.delete_one(doc! {"_id": id}).await?;

        Ok(())
    }

    /// Updates an entity.
    async fn update(&self, entity: Entity) -> Result<Entity, RepositoryError> {
        if entity.id().is_none() {
            return Err(RepositoryError::MissingIdentifier);
        }
        let collection = self.get_collection();

        // Update the entity in the database
        let metadata = collection
            .update_one(
                doc! {"_id": entity.id().unwrap()},
                doc! {"$set": bson::to_document(&entity).map_err(|_| RepositoryError::BsonConversionError)?}
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
