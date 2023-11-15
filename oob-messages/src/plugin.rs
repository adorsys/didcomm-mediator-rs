use axum::Router;
// use super::{didgen, web};
use super::models::create_oob_inv;
use server_plugin::{Plugin, PluginError};

#[derive(Default)]
pub struct OOBMessagesPlugin;

impl Plugin for OOBMessagesPlugin {
    fn name(&self) -> &'static str {
        "oob_messages"
    }

    fn mount(&self) -> Result<(), PluginError> {
        let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
            tracing::error!("SERVER_PUBLIC_DOMAIN env variable required");
            PluginError::InitError
        })?;

        let server_local_port = std::env::var("SERVER_LOCAL_PORT").map_err(|_| {
            tracing::error!("server_local_port env variable required");
            PluginError::InitError
        })?;

        tracing::debug!(
            "Out Of Band Invitation: {}",
            create_oob_inv(&server_public_domain, &server_local_port)
        );

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        Router::new()
    }
}
