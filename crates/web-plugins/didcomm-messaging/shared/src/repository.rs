pub mod entity;

use async_trait::async_trait;
use database::Repository;
use mongodb::{Collection, Database};

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
    fn get_collection(&self) -> Collection<Connection> {
        self.collection.clone()
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
    fn get_collection(&self) -> Collection<RoutedMessage> {
        self.collection.clone()
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use super::*;
    use database::{Repository, RepositoryError};
    use mongodb::bson::{doc, oid::ObjectId, Bson, Document as BsonDocument};
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
        // Implement a dummy get_collection method
        fn get_collection(&self) -> Collection<Connection> {
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

        async fn find_all_by(
            &self,
            filter: BsonDocument,
            limit: Option<i64>,
        ) -> Result<Vec<Connection>, RepositoryError> {
            if let Some(l) = limit {
                if l < 0 {
                    return Ok(vec![]);
                }
            }
            let filter: HashMap<String, Bson> = filter.into_iter().collect();
            Ok(self
                .connections
                .read()
                .unwrap()
                .iter()
                .filter(|c| {
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
                .cloned()
                .collect())
        }

        async fn count_by(&self, filter: BsonDocument) -> Result<usize, RepositoryError> {
            let filter: HashMap<String, Bson> = filter.into_iter().collect();
            Ok(self
                .connections
                .read()
                .unwrap()
                .iter()
                .filter(|c| {
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
                .count())
        }

        // Add new connection to collection
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
        fn get_collection(&self) -> Collection<RoutedMessage> {
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

        async fn find_all_by(
            &self,
            filter: BsonDocument,
            limit: Option<i64>,
        ) -> Result<Vec<RoutedMessage>, RepositoryError> {
            if let Some(l) = limit {
                if l < 0 {
                    return Ok(vec![]);
                }
            }
            let messages = self.messages.read().unwrap();

            // Extract the list of recipient_did values from the filter
            let recipient_dids = filter
                .get("recipient_did")
                .and_then(|value| value.as_document())
                .and_then(|doc| doc.get("$in"))
                .and_then(|value| value.as_array())
                .ok_or(RepositoryError::Generic("invalid filter".to_owned()))?;

            // Convert recipient_dids to a Vec<String>
            let recipient_dids: Vec<String> = recipient_dids
                .iter()
                .filter_map(|value| value.as_str().map(|s| s.to_string()))
                .collect();

            // filter the messages that match any of the recipient_did values
            let mut filtered_messages = messages
                .iter()
                .filter(|msg| recipient_dids.contains(&msg.recipient_did))
                .cloned()
                .collect::<Vec<_>>();

            if let Some(limit) = limit {
                if limit != 0 {
                    filtered_messages.truncate(limit as usize);
                }
            }

            Ok(filtered_messages)
        }

        async fn count_by(&self, filter: BsonDocument) -> Result<usize, RepositoryError> {
            let messages = self.messages.read().unwrap();

            // Extract the list of recipient_did values from the filter
            let recipient_dids = filter
                .get("recipient_did")
                .and_then(|value| value.as_document())
                .and_then(|doc| doc.get("$in"))
                .and_then(|value| value.as_array())
                .ok_or(RepositoryError::Generic("invalid filter".to_owned()))?;

            // Convert recipient_dids to a Vec<String>
            let recipient_dids: Vec<String> = recipient_dids
                .iter()
                .filter_map(|value| value.as_str().map(|s| s.to_string()))
                .collect();

            // Count the messages that match any of the recipient_did values
            let count = messages
                .iter()
                .filter(|msg| recipient_dids.contains(&msg.recipient_did))
                .count();

            Ok(count)
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
            // Find entity to delete
            let pos = self
                .messages
                .read()
                .unwrap()
                .iter()
                .position(|s| s.id == Some(message_id));

            if let Some(pos) = pos {
                self.messages.write().unwrap().remove(pos);
            }
            Ok(())
        }
    }
}
