use super::models::retrieve_or_generate_oob_inv;
use super::models::retrieve_or_generate_qr_image;
use super::web;
use axum::Router;
use did_endpoint::util::filesystem::StdFileSystem;
use server_plugin::{Plugin, PluginError};

#[derive(Default)]
pub struct OOBMessagesPlugin;

impl Plugin for OOBMessagesPlugin {
    fn name(&self) -> &'static str {
        "oob_messages"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
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

        let oob_inv = match retrieve_or_generate_oob_inv(
            &mut fs,
            &server_public_domain,
            &server_local_port,
            &storage_dirpath,
        ) {
            Ok(oob_inv_str) => oob_inv_str,
            Err(err) => {
                tracing::error!("Error retrieving or generating OOB invitation: {}", err);
                return Err(PluginError::InitError);
            }
        };

        tracing::debug!("Out Of Band Invitation: {}", oob_inv);

        match retrieve_or_generate_qr_image(&mut fs, &storage_dirpath, &oob_inv) {
            Ok(_) => {
                // Ignore the QR code image and proceed with error handling
            }
            Err(error_message) => {
                println!(
                    "Error retrieving or generating QR code image: {}",
                    error_message
                );
                tracing::error!("STORAGE_DIRPATH env variable required");
                return Err(PluginError::InitError);
            }
        }

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        web::routes()
    }
}
