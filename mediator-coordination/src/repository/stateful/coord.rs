use async_trait::async_trait;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson, Document as BsonDocument},
    Collection, Database,
};

use crate::{
    model::stateful::coord::entity::Connection,
    repository::traits::{Entity, Repository, RepositoryError},
};

impl Entity for Connection {}

pub struct MongoConnectionRepository {
    collection: Collection<Connection>,
}

impl MongoConnectionRepository {
    pub fn from_db(db: &Database) -> Self {
        Self {
            collection: db.collection("connections"),
        }
    }
}

#[async_trait]
impl Repository<Connection> for MongoConnectionRepository {
    async fn find_all(&self) -> Result<Vec<Connection>, RepositoryError> {
        let mut connections: Vec<Connection> = vec![];

        // Retrieve all connections from the database
        let mut cursor = self.collection.find(None, None).await?;
        while cursor.advance().await? {
            connections.push(cursor.deserialize_current()?);
        }

        Ok(connections)
    }

    async fn find_one(
        &self,
        connection_id: ObjectId,
    ) -> Result<Option<Connection>, RepositoryError> {
        // Query the database for the specified connection ID
        self.find_one_by(doc! {"_id": connection_id}).await
    }

    async fn find_one_by(
        &self,
        filter: BsonDocument,
    ) -> Result<Option<Connection>, RepositoryError> {
        // Query the database for the specified connection ID
        Ok(self
            .collection
            .find_one(filter, None)
            .await?)
    }

    async fn store(&self, connection: Connection) -> Result<Connection, RepositoryError> {
        // Insert the new connection into the database
        let metadata = self.collection.insert_one(connection.clone(), None).await?;

        // Return persisted connection
        Ok(match metadata.inserted_id {
            Bson::ObjectId(oid) => Connection {
                id: Some(oid),
                ..connection
            },
            _ => unreachable!(),
        })
    }

    async fn update(&self, connection: Connection) -> Result<Connection, RepositoryError> {
        if connection.id.is_none() {
            return Err(RepositoryError::MissingIdentifier);
        }

        // Update the connection in the database
        let metadata = self
            .collection
            .update_one(
                doc! {"_id": connection.id.unwrap()},
                doc! {"$set": bson::to_document(&connection).map_err(|_| RepositoryError::BsonConversionError)?},
                None,
            )
            .await?;

        if metadata.matched_count > 0 {
            Ok(connection)
        } else {
            Err(RepositoryError::TargetNotFound)
        }
    }

    async fn delete_one(&self, connection_id: ObjectId) -> Result<(), RepositoryError> {
        // Delete the connection from the database
        let metadata = self
            .collection
            .delete_one(doc! {"_id": connection_id}, None)
            .await?;

        if metadata.deleted_count > 0 {
            Ok(())
        } else {
            Err(RepositoryError::TargetNotFound)
        }
    }
}
