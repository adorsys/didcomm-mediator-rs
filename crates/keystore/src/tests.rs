use crate::NoEncryption;
use crate::{repository::SecretRepository, Error, Keystore};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct MockSecretRepository {
    secrets: RwLock<Vec<(String, Vec<u8>)>>,
}

impl MockSecretRepository {
    pub(crate) fn new(secrets: Vec<(String, Vec<u8>)>) -> Self {
        Self {
            secrets: RwLock::new(secrets),
        }
    }
}

#[async_trait]
impl SecretRepository for MockSecretRepository {
    async fn store(&self, kid: &str, key: &[u8]) -> Result<(), Error> {
        self.secrets
            .write()
            .unwrap()
            .push((kid.to_string(), key.to_owned()));
        Ok(())
    }

    async fn find(&self, kid: &str) -> Result<Option<Vec<u8>>, Error> {
        let secrets = self.secrets.read().unwrap();
        let secret = secrets.iter().find(|(k, _)| k == kid);
        Ok(secret.map(|(_, v)| v.clone()))
    }

    async fn delete(&self, kid: &str) -> Result<(), Error> {
        let mut secrets = self.secrets.write().unwrap();
        let index = secrets.iter().position(|(k, _)| k == kid);
        if let Some(index) = index {
            secrets.remove(index);
        }
        Ok(())
    }
}

impl Keystore {
    /// Create a new key store with mocked repository and encryption backends.
    /// This will be use for testing purposes.
    pub fn with_mock_configs<T>(secrets: Vec<(String, T)>) -> Self
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        let serialized_secrets = secrets
            .into_iter()
            .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()))
            .collect();
        let mock_repository = MockSecretRepository::new(serialized_secrets);
        let encryptor = NoEncryption;
        Self {
            repository: Arc::new(mock_repository),
            encryptor: Arc::new(encryptor),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{encryptor::KeyEncryption, Keystore};
    use async_trait::async_trait;
    use did_utils::jwk::Jwk;
    use serde_json::json;
    use tokio;

    // Simple mock encryptor that reverses the key.
    struct MockEncryptor;

    #[async_trait]
    impl KeyEncryption for MockEncryptor {
        async fn encrypt(&self, key: &[u8]) -> Result<Vec<u8>, crate::Error> {
            let mut key = key.to_vec();
            key.reverse();
            Ok(key)
        }

        async fn decrypt(&self, key: &[u8]) -> Result<Vec<u8>, crate::Error> {
            let mut key = key.to_vec();
            key.reverse();
            Ok(key)
        }
    }

    #[tokio::test]
    async fn test_keystore_flow() {
        let original_key1: Jwk = serde_json::from_value(json!({
            "kty": "OKP",
            "crv": "X25519",
            "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ",
            "d": "0A8SSFkGHg3N9gmVDRnl63ih5fcwtEvnQu9912SVplY"
        }))
        .unwrap();

        let original_key2: Jwk = serde_json::from_value(json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4",
            "d": "fI1u4riKKd99eox08GlThknq-vEJXcKBI28aiUqArLo"
        }))
        .unwrap();

        let dummy_secrets: Vec<(String, Jwk)> = vec![];
        let encryptor = MockEncryptor;
        let keystore = Keystore::with_mock_configs(dummy_secrets).with_encryptor(encryptor);

        keystore.store("key1", &original_key1).await.unwrap();
        keystore.store("key2", &original_key2).await.unwrap();

        // Stored keys should be encrypted and different from the original keys.
        let stored_key1 = keystore.repository.find("key1").await.unwrap().unwrap();
        let stored_key2 = keystore.repository.find("key2").await.unwrap().unwrap();

        let original_key1_bytes = serde_json::to_vec(&original_key1).unwrap();
        let original_key2_bytes = serde_json::to_vec(&original_key2).unwrap();

        assert_ne!(stored_key1, original_key1_bytes);
        assert_ne!(stored_key2, original_key2_bytes);

        // Retrieved keys should match the original keys.
        let retrieved_key1: Jwk = keystore.retrieve("key1").await.unwrap().unwrap();
        let retrieved_key2: Jwk = keystore.retrieve("key2").await.unwrap().unwrap();

        assert_eq!(retrieved_key1, original_key1);
        assert_eq!(retrieved_key2, original_key2);

        // Attempt to retrieve a non-existent key.
        let nonexistent: Option<Jwk> = keystore.retrieve("nonexistent").await.unwrap();
        assert!(nonexistent.is_none());

        // Delete the key1
        keystore.delete("key1").await.unwrap();

        // Attempting to retrieve that key should return None.
        let deleted: Option<Jwk> = keystore.retrieve("key1").await.unwrap();
        assert!(deleted.is_none());

        // Attempting to delete a non-existent key will do nothing.
        let result = keystore.delete("key1").await;
        assert!(result.is_ok());
    }
}
