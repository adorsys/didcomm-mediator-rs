pub mod entity;

use async_trait::async_trait;
use database::Repository;
use mongodb::{Collection, Database};
use std::sync::Arc;
use tokio::sync::RwLock;

use entity::{Connection, RoutedMessage};

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
    fn get_collection(&self) -> Arc<RwLock<Collection<Connection>>> {
        Arc::new(RwLock::new(self.collection.clone()))
    }
}

pub struct MongoMessagesRepository {
    collection: Collection<RoutedMessage>,
}
impl MongoMessagesRepository {
    pub fn from_db(db: &Database) -> Self {
        Self {
            collection: db.collection("messages"),
        }
    }
}
#[async_trait]
impl Repository<RoutedMessage> for MongoMessagesRepository {
    fn get_collection(&self) -> Arc<RwLock<Collection<RoutedMessage>>> {
        Arc::new(RwLock::new(self.collection.clone()))
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use super::*;
    use database::RepositoryError;
    use mongodb::bson::{doc, oid::ObjectId, Bson, Document as BsonDocument};
    use serde_json::json;
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

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
        // Implement a dummy get_collection method
        fn get_collection(&self) -> Arc<tokio::sync::RwLock<Collection<Connection>>> {
            // In-memory, we don't have an actual collection, but we can create a dummy Arc<Mutex> for compatibility.
            unimplemented!("This is a mock repository, no real collection exists.")
        }
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

        async fn store(&self, mut connection: Connection) -> Result<Connection, RepositoryError> {
            connection.id = Some(ObjectId::new());
            self.connections.write().unwrap().push(connection.clone());
            Ok(connection)
        }

        async fn update(&self, connection: Connection) -> Result<Connection, RepositoryError> {
            if connection.id.is_none() {
                return Err(RepositoryError::MissingIdentifier);
            }

            let mut connections = self.connections.write().unwrap();
            if let Some(pos) = connections.iter().position(|c| c.id == connection.id) {
                connections[pos] = connection.clone();
                Ok(connection)
            } else {
                Err(RepositoryError::TargetNotFound)
            }
        }

        async fn delete_one(&self, connection_id: ObjectId) -> Result<(), RepositoryError> {
            let mut connections = self.connections.write().unwrap();
            if let Some(pos) = connections.iter().position(|c| c.id == Some(connection_id)) {
                connections.remove(pos);
                Ok(())
            } else {
                Err(RepositoryError::TargetNotFound)
            }
        }
    }

    pub struct MockMessagesRepository {
        messages: RwLock<Vec<RoutedMessage>>,
    }

    impl MockMessagesRepository {
        pub fn from(messages: Vec<RoutedMessage>) -> Self {
            Self {
                messages: RwLock::new(messages),
            }
        }
    }

    #[async_trait]
    impl Repository<RoutedMessage> for MockMessagesRepository {
        // Implement a dummy get_collection method
        fn get_collection(&self) -> Arc<tokio::sync::RwLock<Collection<RoutedMessage>>> {
            // In-memory, we don't have an actual collection, but we can create a dummy Arc<Mutex> for compatibility.
            unimplemented!("This is a mock repository, no real collection exists.")
        }

        async fn find_all(&self) -> Result<Vec<RoutedMessage>, RepositoryError> {
            Ok(self.messages.read().unwrap().clone())
        }

        async fn find_one(
            &self,
            message_id: ObjectId,
        ) -> Result<Option<RoutedMessage>, RepositoryError> {
            self.find_one_by(doc! {"_id": message_id}).await
        }

        async fn find_one_by(
            &self,
            filter: BsonDocument,
        ) -> Result<Option<RoutedMessage>, RepositoryError> {
            let filter: HashMap<String, Bson> = filter.into_iter().collect();
            Ok(self
                .messages
                .read()
                .unwrap()
                .iter()
                .find(|m| {
                    if let Some(id) = filter.get("_id") {
                        if json!(m.id) != json!(id) {
                            return false;
                        }
                    }
                    true
                })
                .cloned())
        }

        async fn store(&self, messages: RoutedMessage) -> Result<RoutedMessage, RepositoryError> {
            self.messages.write().unwrap().push(messages.clone());
            Ok(messages)
        }

        async fn update(&self, messages: RoutedMessage) -> Result<RoutedMessage, RepositoryError> {
            let mut messages_list = self.messages.write().unwrap();
            if let Some(pos) = messages_list.iter().position(|m| m.id == messages.id) {
                messages_list[pos] = messages.clone();
                Ok(messages)
            } else {
                Err(RepositoryError::TargetNotFound)
            }
        }

        async fn delete_one(&self, message_id: ObjectId) -> Result<(), RepositoryError> {
            let mut messages_list = self.messages.write().unwrap();
            if let Some(pos) = messages_list.iter().position(|m| m.id == Some(message_id)) {
                messages_list.remove(pos);
                Ok(())
            } else {
                Err(RepositoryError::TargetNotFound)
            }
        }
    }
}
