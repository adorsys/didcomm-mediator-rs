pub(crate) mod mongodb;

use async_trait::async_trait;

use crate::Error;

/// Abstract interface for secret storage backends.
#[async_trait]
pub trait SecretRepository: Send + Sync {
    async fn store(&self, kid: &str, key: &[u8]) -> Result<(), Error>;
    async fn find(&self, kid: &str) -> Result<Option<Vec<u8>>, Error>;
    async fn delete(&self, kid: &str) -> Result<(), Error>;
}
