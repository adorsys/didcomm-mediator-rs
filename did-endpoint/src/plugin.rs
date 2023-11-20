use super::{didgen, web};
use axum::Router;
use server_plugin::{Plugin, PluginError};

#[derive(Default)]
pub struct DidEndpointPlugin;

impl Plugin for DidEndpointPlugin {
    fn name(&self) -> &'static str {
        "did_endpoint"
    }

    fn mount(&self) -> Result<(), PluginError> {
        let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|_| {
            tracing::error!("STORAGE_DIRPATH env variable required");
            PluginError::InitError
        })?;

        if didgen::validate_diddoc(&storage_dirpath).is_err() {
            tracing::debug!("diddoc validation failed, will generate one");

            let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
                tracing::error!("SERVER_PUBLIC_DOMAIN env variable required");
                PluginError::InitError
            })?;

            didgen::didgen(&storage_dirpath, &server_public_domain).map_err(|_| {
                tracing::error!("failed to generate an initial keystore and its DID document");
                PluginError::InitError
            })?;
        };

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        web::routes()
    }
}
