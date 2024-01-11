use async_trait::async_trait;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson, Document as BsonDocument},
    Collection, Database,
};

use crate::{
    model::stateful::coord::entity::{Connection, Secrets},
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
        Ok(self.collection.find_one(filter, None).await?)
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

impl Entity for Secrets {} 

pub struct MongoSecretsRepository {
    collection: Collection<Secrets>, // Use the Secrets entity for the collection
}

impl MongoSecretsRepository {
    pub fn from_db(db: &Database) -> Self {
        Self {
            collection: db.collection("secrets"), // Use the "secrets" collection
        }
    }
}

#[async_trait]
impl Repository<Secrets> for MongoSecretsRepository {
    async fn find_all(&self) -> Result<Vec<Secrets>, RepositoryError> {
        let mut secrets: Vec<Secrets> = vec![];

        // Retrieve all secrets from the database
        let mut cursor = self.collection.find(None, None).await?;
        while cursor.advance().await? {
            secrets.push(cursor.deserialize_current()?);
        }

        Ok(secrets)
    }

    async fn find_one(
        &self,
        secrets_id: ObjectId,
    ) -> Result<Option<Secrets>, RepositoryError> {
        // Query the database for the specified secrets ID
        self.find_one_by(doc! {"_id": secrets_id}).await
    }

    async fn find_one_by(
        &self,
        filter: BsonDocument,
    ) -> Result<Option<Secrets>, RepositoryError> {
        // Query the database for the specified secrets ID
        Ok(self.collection.find_one(filter, None).await?)
    }

    async fn store(&self, secrets: Secrets) -> Result<Secrets, RepositoryError> {
        // Insert the new secrets into the database
        let metadata = self.collection.insert_one(secrets.clone(), None).await?;

        // Return persisted secrets
        Ok(match metadata.inserted_id {
            Bson::ObjectId(oid) => Secrets {
                id: oid,
                ..secrets
            },
            _ => unreachable!(),
        })
    }

    async fn update(&self, secrets: Secrets) -> Result<Secrets, RepositoryError> {
        if secrets.id == ObjectId::default() {
            return Err(RepositoryError::MissingIdentifier);
        }

        // Update the secrets in the database
        let metadata = self
            .collection
            .update_one(
                doc! {"_id": &secrets.id},
                doc! {"$set": bson::to_document(&secrets).map_err(|_| RepositoryError::BsonConversionError)?},
                None,
            )
            .await?;

        if metadata.matched_count > 0 {
            Ok(secrets)
        } else {
            Err(RepositoryError::TargetNotFound)
        }

    }

    async fn delete_one(&self, secrets_id: ObjectId) -> Result<(), RepositoryError> {
        // Delete the secrets from the database
        let metadata = self
            .collection
            .delete_one(doc! {"_id": secrets_id}, None)
            .await?;

        if metadata.deleted_count > 0 {
            Ok(())
        } else {
            Err(RepositoryError::TargetNotFound)
        }
    }
}
#[cfg(test)]
pub mod tests {
    use super::*;

    use serde_json::json;
    use std::{collections::HashMap, sync::RwLock};

    pub struct MockConnectionRepository {
        connections: RwLock<Vec<Connection>>,
    }

    impl MockConnectionRepository {
        pub fn from(connections: Vec<Connection>) -> Self {
            Self {
                connections: RwLock::new(connections),
            }
        }
    }

    #[async_trait]
    impl Repository<Connection> for MockConnectionRepository {
        async fn find_all(&self) -> Result<Vec<Connection>, RepositoryError> {
            Ok(self.connections.read().unwrap().clone())
        }

        async fn find_one(
            &self,
            connection_id: ObjectId,
        ) -> Result<Option<Connection>, RepositoryError> {
            self.find_one_by(doc! {"_id": connection_id}).await
        }

        async fn find_one_by(
            &self,
            filter: BsonDocument,
        ) -> Result<Option<Connection>, RepositoryError> {
            let filter: HashMap<String, Bson> = filter.into_iter().collect();

            Ok(self
                .connections
                .read()
                .unwrap()
                .iter()
                .find(|c| {
                    if let Some(id) = filter.get("_id") {
                        if json!(c.id) != json!(id) {
                            return false;
                        }
                    }

                    if let Some(client_did) = filter.get("client_did") {
                        if json!(c.client_did) != json!(client_did) {
                            return false;
                        }
                    }

                    true
                })
                .cloned())
        }

        async fn store(&self, connection: Connection) -> Result<Connection, RepositoryError> {
            // Generate a new ID for the entity
            let connection = Connection {
                id: Some(ObjectId::new()),
                ..connection
            };

            // Add new connection to collection
            self.connections.write().unwrap().push(connection.clone());

            // Return added connection
            Ok(connection)
        }

        async fn update(&self, connection: Connection) -> Result<Connection, RepositoryError> {
            if connection.id.is_none() {
                return Err(RepositoryError::MissingIdentifier);
            }

            // Find entity to update
            let pos = self
                .connections
                .read()
                .unwrap()
                .iter()
                .position(|c| c.id == connection.id);

            if let Some(pos) = pos {
                self.connections.write().unwrap()[pos] = connection.clone();

                Ok(connection)
            } else {
                Err(RepositoryError::TargetNotFound)
            }
        }

        async fn delete_one(&self, connection_id: ObjectId) -> Result<(), RepositoryError> {
            // Find entity to delete
            let pos = self
                .connections
                .read()
                .unwrap()
                .iter()
                .position(|c| c.id.as_ref().unwrap() == &connection_id);

            if let Some(pos) = pos {
                self.connections.write().unwrap().remove(pos);
                Ok(())
            } else {
                Err(RepositoryError::TargetNotFound)
            }
        }
    }
}
