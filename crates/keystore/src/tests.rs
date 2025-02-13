use crate::{repository::SecretRepository, Error, ErrorKind};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::RwLock;

#[derive(Default)]
pub(crate) struct MockSecretRepository<T>
where
    T: Serialize + DeserializeOwned,
{
    secrets: RwLock<Vec<(String, T)>>,
}

impl<T: Serialize + DeserializeOwned> MockSecretRepository<T> {
    pub(crate) fn new(secrets: Vec<(String, T)>) -> Self {
        Self {
            secrets: RwLock::new(secrets),
        }
    }
}

#[async_trait]
impl<T: Serialize + DeserializeOwned + Send + Sync> SecretRepository for MockSecretRepository<T> {
    async fn store(&self, kid: &str, key: &[u8]) -> Result<(), Error> {
        let key = serde_json::from_slice(key)?;
        self.secrets.write().unwrap().push((kid.to_string(), key));

        Ok(())
    }

    async fn find(&self, kid: &str) -> Result<Option<Vec<u8>>, Error> {
        let secrets = self.secrets.read().unwrap();
        let secret = secrets.iter().find(|(k, _)| k == kid);
        match secret {
            Some((_, secret)) => {
                let secret = serde_json::to_vec(secret)?;
                Ok(Some(secret))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, kid: &str) -> Result<(), Error> {
        let mut secrets = self.secrets.write().unwrap();
        let index = secrets.iter().position(|(k, _)| k == kid);
        match index {
            Some(index) => {
                secrets.remove(index);
                Ok(())
            }
            None => Err(Error::msg(
                ErrorKind::RepositoryFailure,
                format!("Secret with kid {kid} not found"),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Keystore;
    use did_utils::jwk::Jwk;
    use serde_json::json;
    use tokio;

    fn secret1() -> Jwk {
        serde_json::from_value(json!({
            "kty": "OKP",
            "crv": "X25519",
            "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ",
            "d": "0A8SSFkGHg3N9gmVDRnl63ih5fcwtEvnQu9912SVplY"
        }))
        .unwrap()
    }

    fn secret2() -> Jwk {
        serde_json::from_value(json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4",
            "d": "fI1u4riKKd99eox08GlThknq-vEJXcKBI28aiUqArLo"
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn test_keystore_flow() {
        let jwk1 = secret1();
        let jwk2 = secret2();

        let dummy_secrets: Vec<(String, Jwk)> = vec![];
        let keystore = Keystore::with_mock_configs(dummy_secrets);

        keystore.store("key1", &jwk1).await.unwrap();
        keystore.store("key2", &jwk2).await.unwrap();

        // Retrieve key1.
        let key1 = keystore.retrieve("key1").await.unwrap();
        assert_eq!(key1, Some(jwk1));

        // Retrieve key2.
        let key2 = keystore.retrieve("key2").await.unwrap();
        assert_eq!(key2, Some(jwk2));

        // Attempt to retrieve a non-existent key.
        let nonexistent: Option<Jwk> = keystore.retrieve("nonexistent").await.unwrap();
        assert!(nonexistent.is_none());

        // Delete the key1
        keystore.delete("key1").await.unwrap();

        // Attempting to retrieve that key should return None.
        let deleted: Option<Jwk> = keystore.retrieve("key1").await.unwrap();
        assert!(deleted.is_none());

        // Attempting to delete a non-existent key should return an error.
        let result = keystore.delete("key1").await;
        assert!(result.is_err());
    }
}
