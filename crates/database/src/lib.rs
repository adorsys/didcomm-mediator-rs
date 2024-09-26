use async_trait::async_trait;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson, Document as BsonDocument},
    error::Error as MongoError,
    options::FindOptions,
    Collection,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

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

/// Definition of a trait for repository operations.
#[async_trait]
pub trait Repository<Entity>: Sync + Send
where
    Entity: Identifiable
        + Sized
        + Serialize
        + Clone
        + Send
        + Sync
        + for<'de> Deserialize<'de>
        + Unpin
        + 'static,
{
    fn get_collection(&self) -> Arc<Mutex<Collection<Entity>>>;

    async fn find_all(&self) -> Result<Vec<Entity>, RepositoryError> {
        let mut entities = Vec::new();
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let mut cursor = collection.lock().await.find(None, None).await?;
        while cursor.advance().await? {
            entities.push(cursor.deserialize_current()?);
        }

        Ok(entities)
    }

    async fn find_one(&self, message_id: ObjectId) -> Result<Option<Entity>, RepositoryError> {
        self.find_one_by(doc! {"_id": message_id}).await
    }

    async fn find_one_by(&self, filter: BsonDocument) -> Result<Option<Entity>, RepositoryError> {
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.lock().await;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn store(&self, mut entity: Entity) -> Result<Entity, RepositoryError> {
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.lock().await;

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
        let collection = collection.lock().await;

        // Retrieve all entities from the database
        let mut cursor = collection.find(filter, find_options).await?;
        while cursor.advance().await? {
            entities.push(cursor.deserialize_current()?);
        }

        Ok(entities)
    }

    async fn delete_one(&self, message_id: ObjectId) -> Result<(), RepositoryError> {
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.lock().await;

        // Delete the entity from the database
        collection
            .delete_one(doc! {"_id": message_id}, None)
            .await?;

        Ok(())
    }

    async fn update(&self, entity: Entity) -> Result<Entity, RepositoryError> {
        if entity.id().is_none() {
            return Err(RepositoryError::MissingIdentifier);
        }
        let collection = self.get_collection();

        // Lock the Mutex and get the Collection
        let collection = collection.lock().await;

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
