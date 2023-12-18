use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A trait representing an abstract resource.
/// Any type implementing this trait should also implement `Serialize`.
pub trait Entity: Sized + Serialize {}

// Definition of custom errors for repository operations
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

// Definition of a trait for repository operations
#[async_trait]
pub trait Repository<Entity, EntityKeyType>: Sync + Send {
    // Retrieves all entities.
    async fn find_all(&self) -> Result<Vec<Entity>, RepositoryError>;

    // Retrieves a single entity by its identifier.
    async fn find_one(&self, entity_id: EntityKeyType) -> Result<Option<Entity>, RepositoryError>;

    // Stores a new entity.
    async fn store(&self, entity: Entity) -> Result<Entity, RepositoryError>;

    // Updates an existing entity.
    async fn update(&self, entity: Entity) -> Result<Entity, RepositoryError>;

    // Deletes a single entity by its identifier.
    async fn delete_one(&self, entity_id: EntityKeyType) -> Result<(), RepositoryError>;
}
