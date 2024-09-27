use std::sync::Arc;
use async_trait::async_trait;
use database::{Repository, RepositoryError};
use mongodb::{

    bson::{doc, oid::ObjectId, Document as BsonDocument},
    Collection, Database,
};
use tokio::sync::Mutex;

use crate::model::stateful::entity::{Connection, RoutedMessage, Secrets};

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
    fn get_collection(&self) -> Arc<Mutex<Collection<Connection>>> {
        Arc::new(Mutex::new(self.collection.clone()))
    }
}

pub struct MongoSecretsRepository {
    collection: Collection<Secrets>,
}

impl MongoSecretsRepository {
    pub fn from_db(db: &Database) -> Self {
        Self {
            collection: db.collection("secrets"),
        }
    }
}

#[async_trait]
impl Repository<Secrets> for MongoSecretsRepository {
    fn get_collection(&self) -> Arc<Mutex<Collection<Secrets>>> {
        Arc::new(Mutex::new(self.collection.clone()))
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
    fn get_collection(&self) -> Arc<Mutex<Collection<RoutedMessage>>> {
        Arc::new(Mutex::new(self.collection.clone()))
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use mongodb::bson::Bson;
    use serde_json::json;
    use std::{collections::HashMap, sync::{Arc, RwLock}};
    use tokio::sync::Mutex;

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
        fn get_collection(&self) -> Arc<Mutex<Collection<Connection>>> {
            // In-memory, we don't have an actual collection, but we can create a dummy Arc<Mutex> for compatibility.
            unimplemented!("This is a mock repository, no real collection exists.")
        }
        async fn find_all(&self) -> Result<Vec<Connection>, RepositoryError> {
            Ok(self.connections.read().unwrap().clone())
        }

        async fn find_one(&self, connection_id: ObjectId) -> Result<Option<Connection>, RepositoryError> {
            self.find_one_by(doc! {"_id": connection_id}).await
        }

        async fn find_one_by(&self, filter: BsonDocument) -> Result<Option<Connection>, RepositoryError> {
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

    pub struct MockSecretsRepository {
        secrets: RwLock<Vec<Secrets>>,
    }

    impl MockSecretsRepository {
        pub fn from(secrets: Vec<Secrets>) -> Self {
            Self {
                secrets: RwLock::new(secrets),
            }
        }
    }

    #[async_trait]
    impl Repository<Secrets> for MockSecretsRepository {
        // Implement a dummy get_collection method
        fn get_collection(&self) -> Arc<Mutex<Collection<Secrets>>> {
            // In-memory, we don't have an actual collection, but we can create a dummy Arc<Mutex> for compatibility.
            unimplemented!("This is a mock repository, no real collection exists.")
        }

        async fn find_all(&self) -> Result<Vec<Secrets>, RepositoryError> {
            Ok(self.secrets.read().unwrap().clone())
        }

        async fn find_one(&self, secrets_id: ObjectId) -> Result<Option<Secrets>, RepositoryError> {
            self.find_one_by(doc! {"_id": secrets_id}).await
        }

        async fn find_one_by(&self, filter: BsonDocument) -> Result<Option<Secrets>, RepositoryError> {
            let filter: HashMap<String, Bson> = filter.into_iter().collect();
            Ok(self
                .secrets
                .read()
                .unwrap()
                .iter()
                .find(|s| {
                    if let Some(id) = filter.get("_id") {
                        if json!(s.id) != json!(id) {
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
        fn get_collection(&self) -> Arc<Mutex<Collection<RoutedMessage>>> {
            // In-memory, we don't have an actual collection, but we can create a dummy Arc<Mutex> for compatibility.
            unimplemented!("This is a mock repository, no real collection exists.")
        }

        async fn find_all(&self) -> Result<Vec<RoutedMessage>, RepositoryError> {
            Ok(self.messages.read().unwrap().clone())
        }

        async fn find_one(&self, message_id: ObjectId) -> Result<Option<RoutedMessage>, RepositoryError> {
            self.find_one_by(doc! {"_id": message_id}).await
        }

        async fn find_one_by(&self, filter: BsonDocument) -> Result<Option<RoutedMessage>, RepositoryError> {
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
            let filter: HashMap<String, Bson> = filter.into_iter().collect();
            Ok(self
                .messages
                .read()
                .unwrap()
                .iter()
                .filter(|s| {
                    if let Some(id) = filter.get("_id") {
                        if json!(s.id) != json!(id) {
                            return false;
                        }
                    }

                    true
                })
                .cloned()
                .collect())
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