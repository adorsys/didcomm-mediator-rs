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
        match self.client.describe_secret().secret_id(name).send().await {
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

        match self.cache.get_secret_value(name, None, None, false).await {
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
        self.client.delete_secret().secret_id(name).send().await?;

        // Invalidate cache by refreshing the secret
        let _ = self.cache.get_secret_value(name, None, None, true).await;
        Ok(())
    }
}
