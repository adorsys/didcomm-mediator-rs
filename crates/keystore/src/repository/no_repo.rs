use async_trait::async_trait;

use crate::Error;

use super::SecretRepository;

pub(crate) struct NoRepository;

#[async_trait]
impl SecretRepository for NoRepository {
    async fn store(&self, _kid: &str, _key: &[u8]) -> Result<(), Error> {
        Ok(())
    }

    async fn find(&self, _kid: &str) -> Result<Option<Vec<u8>>, Error> {
        Ok(None)
    }

    async fn delete(&self, _kid: &str) -> Result<(), Error> {
        Ok(())
    }
}
