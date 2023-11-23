use crate::{util, web};

use axum::Router;
use did_endpoint::{didgen, util::filesystem::StdFileSystem};
use server_plugin::{Plugin, PluginError};

#[derive(Default)]
pub struct MediatorCoordinationPlugin;

impl Plugin for MediatorCoordinationPlugin {
    fn name(&self) -> &'static str {
        "mediator_coordination"
    }

    fn mount(&self) -> Result<(), PluginError> {
        let _public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
            tracing::error!("SERVER_PUBLIC_DOMAIN env variable required");
            PluginError::InitError
        })?;

        let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|_| {
            tracing::error!("STORAGE_DIRPATH env variable required");
            PluginError::InitError
        })?;

        // Expect DID document from file system
        if didgen::validate_diddoc(&storage_dirpath).is_err() {
            tracing::error!("diddoc validation failed; is plugin did-endpoint mounted?");
            return Err(PluginError::InitError);
        }

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        let msg = "This should not occur following successful mounting.";
        let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").expect(msg);
        let storage_dirpath = std::env::var("STORAGE_DIRPATH").expect(msg);

        let mut fs = StdFileSystem;
        let diddoc = util::read_diddoc(&fs, &storage_dirpath).expect(msg);
        let keystore = util::read_keystore(&mut fs, &storage_dirpath).expect(msg);

        web::routes(public_domain, diddoc, keystore)
    }
}
