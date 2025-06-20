use async_trait::async_trait;
use database::{Identifiable, Repository};
use did_utils::didcore::Document as DidDocument;
use mongodb::{bson::oid::ObjectId, Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediatorDidDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub diddoc: DidDocument,
}

impl Identifiable for MediatorDidDocument {
    fn id(&self) -> Option<ObjectId> {
        self.id
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}

pub struct DidDocumentRepository {
    collection: Collection<MediatorDidDocument>,
}

impl DidDocumentRepository {
    pub fn from_db(db: &Database) -> Self {
        Self {
            collection: db.collection("mediator_diddoc"),
        }
    }
}

#[async_trait]
impl Repository<MediatorDidDocument> for DidDocumentRepository {
    fn get_collection(&self) -> Collection<MediatorDidDocument> {
        self.collection.clone()
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use super::*;
    use async_trait::async_trait;
    use database::{Repository, RepositoryError};
    use mongodb::{bson::Document as BsonDocument, Collection};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Clone, Default)]
    pub struct MockDidDocumentRepository {
        diddoc: Arc<RwLock<Option<MediatorDidDocument>>>,
    }

    impl MockDidDocumentRepository {
        pub fn new() -> Self {
            Self {
                diddoc: Arc::new(RwLock::new(None)),
            }
        }
    }

    #[async_trait]
    impl Repository<MediatorDidDocument> for MockDidDocumentRepository {
        fn get_collection(&self) -> Collection<MediatorDidDocument> {
            unimplemented!("This is a mock repository, no real collection exists.")
        }

        async fn find_one_by(
            &self,
            _filter: BsonDocument,
        ) -> Result<Option<MediatorDidDocument>, RepositoryError> {
            let diddoc = self.diddoc.read().await;
            Ok(diddoc.clone())
        }

        async fn store(
            &self,
            diddoc: MediatorDidDocument,
        ) -> Result<MediatorDidDocument, RepositoryError> {
            let mut lock = self.diddoc.write().await;
            *lock = Some(diddoc.clone());
            Ok(diddoc)
        }
    }
}
