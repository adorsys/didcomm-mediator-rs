use axum::Router;
use super::web;
use super::models::retrieve_or_generate_oob_inv;
use super::models::retrieve_or_generate_qr_image;
use server_plugin::{Plugin, PluginError};
use did_endpoint::util::filesystem::StdFileSystem;

#[derive(Default)]
pub struct OOBMessagesPlugin;

impl Plugin for OOBMessagesPlugin {
    fn name(&self) -> &'static str {
        "oob_messages"
    }

    fn mount(&self) -> Result<(), PluginError> {
        let mut fs = StdFileSystem;

        let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
            tracing::error!("SERVER_PUBLIC_DOMAIN env variable required");
            PluginError::InitError
        })?;

        let server_local_port = std::env::var("SERVER_LOCAL_PORT").map_err(|_| {
            tracing::error!("SERVER_LOCAL_PORT env variable required");
            PluginError::InitError
        })?;

        let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|_| {
            tracing::error!("STORAGE_DIRPATH env variable required");
            PluginError::InitError
        })?;

        let oob_inv = retrieve_or_generate_oob_inv(&mut fs, &server_public_domain, &server_local_port, &storage_dirpath);
        tracing::debug!(
            "Out Of Band Invitation: {}",
            oob_inv
        );

        retrieve_or_generate_qr_image(&storage_dirpath, &oob_inv);

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        web::routes()
    }
}
