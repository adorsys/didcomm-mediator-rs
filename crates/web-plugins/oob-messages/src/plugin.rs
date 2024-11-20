use super::{
    models::{retrieve_or_generate_oob_inv, retrieve_or_generate_qr_image},
    web,
};
use axum::Router;
use filesystem::StdFileSystem;
use plugin_api::{Plugin, PluginError};

#[derive(Default)]
pub struct OOBMessages;

impl Plugin for OOBMessages {
    fn name(&self) -> &'static str {
        "oob_messages"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        let mut fs = StdFileSystem;

        let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
            PluginError::InitError("SERVER_PUBLIC_DOMAIN env variable required".to_owned())
        })?;

        let server_local_port = std::env::var("SERVER_LOCAL_PORT").map_err(|_| {
            PluginError::InitError("SERVER_LOCAL_PORT env variable required".to_owned())
        })?;

        let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|_| {
            PluginError::InitError("STORAGE_DIRPATH env variable required".to_owned())
        })?;

        let oob_inv = retrieve_or_generate_oob_inv(
            &mut fs,
            &server_public_domain,
            &server_local_port,
            &storage_dirpath,
        )
        .map_err(|e| {
            PluginError::InitError(format!(
                "Error retrieving or generating OOB invitation: {e}"
            ))
        })?;

        tracing::debug!("Out Of Band Invitation: {}", oob_inv);

        let _ =
            retrieve_or_generate_qr_image(&mut fs, &storage_dirpath, &oob_inv).map_err(|e| {
                PluginError::InitError(format!("Error retrieving or generating QR code image: {e}"))
            })?;

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        web::routes()
    }
}
