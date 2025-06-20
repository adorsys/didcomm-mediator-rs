pub(crate) mod aws;
pub(crate) mod mongodb;
pub(crate) mod no_repo;

use async_trait::async_trait;

use crate::Error;

/// Abstract interface for secret storage backends.
#[async_trait]
pub trait SecretRepository: Send + Sync {
    /// Store a given secret's bytes in the repository.
    async fn store(&self, kid: &str, key: &[u8]) -> Result<(), Error>;

    /// Retrieve a secret's bytes from the repository.
    async fn find(&self, kid: &str) -> Result<Option<Vec<u8>>, Error>;

    /// Delete a secret from the repository.
    async fn delete(&self, kid: &str) -> Result<(), Error>;
}
