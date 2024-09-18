use async_trait::async_trait;
use mongodb::{
    bson::{oid::ObjectId, Document as BsonDocument},
    error::Error as MongoError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
/// A trait representing an abstract resource.
/// Any type implementing this trait should also implement `Serialize`.

/// Definition of custom errors for repository operations
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

/// Definition of a trait for repository operations
#[async_trait]
pub trait Repository<Entity>: Sync + Send
where
    Entity: Sized + Serialize,
{
    /// Retrieves all entities.
    async fn find_all(&self) -> Result<Vec<Entity>, RepositoryError>;

    /// Retrieves a single entity by its identifier.
    async fn find_one(&self, entity_id: ObjectId) -> Result<Option<Entity>, RepositoryError>;

    /// Retrieves a single entity by filter.
    async fn find_one_by(&self, filter: BsonDocument) -> Result<Option<Entity>, RepositoryError>;

    /// Stores a new entity.
    async fn store(&self, entity: Entity) -> Result<Entity, RepositoryError>;

    /// Updates an existing entity.
    async fn update(&self, entity: Entity) -> Result<Entity, RepositoryError>;

    /// Deletes a single entity by its identifier.
    async fn delete_one(&self, entity_id: ObjectId) -> Result<(), RepositoryError>;

}

impl From<MongoError> for RepositoryError {
    fn from(error: MongoError) -> Self {
        RepositoryError::Generic(error.to_string())
    }
}
