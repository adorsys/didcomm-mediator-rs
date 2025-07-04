use std::{num::NonZeroUsize, time::Duration};

use async_trait::async_trait;
use aws_config::SdkConfig;
use aws_sdk_secretsmanager::{
    operation::get_secret_value::GetSecretValueError, Client as SecretsClient,
    Config as SecretsConfig,
};
use aws_secretsmanager_caching::SecretsManagerCachingClient as SecretsCacheClient;
use eyre::eyre;
use tracing::warn;

use crate::{Error, ErrorKind};

use super::SecretRepository;

/// Type used for AWS Secrets Manager operations
pub(crate) struct AwsSecretsManager {
    client: SecretsClient,
    cache: SecretsCacheClient,
}

impl AwsSecretsManager {
    /// Create a new instance of [AwsSecretsManager] with the given AWS SDK config
    pub async fn new(config: &SdkConfig) -> Self {
        let client = SecretsClient::new(config);
        // Cache size: 100 and a TTL of 5 minutes
        let cache = SecretsCacheClient::from_builder(
            SecretsConfig::from(config).to_builder(),
            NonZeroUsize::new(100).unwrap(),
            Duration::from_secs(300),
            true,
        )
        .await
        .unwrap();

        Self { client, cache }
    }
}

#[async_trait]
impl SecretRepository for AwsSecretsManager {
    async fn store(&self, name: &str, data: &[u8]) -> Result<(), Error> {
        use aws_sdk_secretsmanager::error::SdkError;

        // Store a secret only if it does not already exist
        let name = sanitize_secret_name(name);
        match self.client.describe_secret().secret_id(&name).send().await {
            Ok(_) => {
                warn!("Secret {name} already exists. Skipping...");
                Ok(())
            }
            Err(SdkError::ServiceError(err)) if err.err().is_resource_not_found_exception() => {
                let secret = String::from_utf8_lossy(data).to_string();
                // Secret does not exist, try to create it
                self.client
                    .create_secret()
                    .name(name)
                    .secret_string(secret)
                    .send()
                    .await?;
                Ok(())
            }
            Err(sdk_err) => Err(sdk_err.into()),
        }
    }

    async fn find(&self, name: &str) -> Result<Option<Vec<u8>>, Error> {
        use aws_sdk_secretsmanager::error::SdkError;

        let name = sanitize_secret_name(name);
        match self.cache.get_secret_value(&name, None, None, false).await {
            Ok(value) => Ok(value.secret_string.map(|s| s.into_bytes())),
            Err(err) => {
                // Check for ResourceNotFoundException
                if let Some(SdkError::ServiceError(service_err)) =
                    err.downcast_ref::<SdkError<GetSecretValueError>>()
                {
                    if service_err.err().is_resource_not_found_exception() {
                        return Ok(None);
                    }
                }
                Err(Error::msg(ErrorKind::RepositoryFailure, eyre!("{err}")))
            }
        }
    }

    async fn delete(&self, name: &str) -> Result<(), Error> {
        let name = sanitize_secret_name(name);
        self.client.delete_secret().secret_id(&name).send().await?;

        // Invalidate cache by refreshing the secret
        let _ = self.cache.get_secret_value(&name, None, None, true).await;
        Ok(())
    }
}

// Replaces characters in a secret ID that are not supported by AWS Secrets Manager.
//
// This function replaces the following characters:
// - `:` is replaced with `_colon_`
// - `#` is replaced with `_hash_`
fn sanitize_secret_name(id: &str) -> String {
    id.replace(':', "_colon_").replace('#', "_hash_")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_secret_name(id: &str) -> String {
        id.replace("_hash_", "#").replace("_colon_", ":")
    }

    #[test]
    fn test_sanitize_secret_name_no_special_chars() {
        let original_id = "example_good/secret@name.+=";
        let encoded_id = sanitize_secret_name(original_id);
        let decoded_id = decode_secret_name(&encoded_id);

        assert_eq!(encoded_id, "example_good/secret@name.+=");
        assert_eq!(decoded_id, original_id);
    }

    #[test]
    fn test_sanitize_secret_name_with_special_chars() {
        let original_id = "example:bad:secret#name.+=";
        let encoded_id = sanitize_secret_name(original_id);
        let decoded_id = decode_secret_name(&encoded_id);

        assert_eq!(encoded_id, "example_colon_bad_colon_secret_hash_name.+=");
        assert_eq!(decoded_id, original_id);
    }
}
