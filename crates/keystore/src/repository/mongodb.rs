use async_trait::async_trait;
use mongodb::{bson::doc, Collection};
use once_cell::sync::OnceCell;
use tokio::{runtime::Handle, task::block_in_place};

use crate::Secrets;

use super::SecretRepository;

/// MongoDB implementation of the secret repository.
pub(crate) struct MongoSecretRepository {
    collection: Collection<Secrets>,
}

impl MongoSecretRepository {
    /// Create a new instance of the MongoDB secret repository.
    ///
    /// The secret collection will be initialized once.
    pub(crate) fn new() -> Self {
        static SECRETS_COLLECTION: OnceCell<Collection<Secrets>> = OnceCell::new();
        let db = database::get_or_init_database();
        let collection = SECRETS_COLLECTION
            .get_or_init(|| {
                let task = async move { db.collection::<Secrets>("secrets").clone() };
                block_in_place(|| Handle::current().block_on(task))
            })
            .clone();

        Self { collection }
    }
}

#[async_trait]
impl SecretRepository for MongoSecretRepository {
    async fn store(&self, kid: &str, key: &[u8]) -> Result<(), crate::Error> {
        let secret = Secrets {
            id: None,
            kid: kid.to_string(),
            secret_material: key.to_vec(),
        };
        self.collection.insert_one(secret).await?;
        Ok(())
    }

    async fn find(&self, kid: &str) -> Result<Option<Vec<u8>>, crate::Error> {
        let result = self.collection.find_one(doc! { "kid": kid }).await?;
        if let Some(secret) = result {
            Ok(Some(secret.secret_material))
        } else {
            Ok(None)
        }
    }

    #[inline]
    async fn delete(&self, kid: &str) -> Result<(), crate::Error> {
        self.collection.delete_one(doc! { "kid": kid }).await?;
        Ok(())
    }
}
