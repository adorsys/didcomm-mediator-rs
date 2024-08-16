use crate::{util, web};

use axum::Router;
use keystore::filesystem::StdFileSystem;
use plugin_api::{Plugin, PluginError};

#[derive(Default)]
pub struct MediatorCoordinationPlugin;

impl Plugin for MediatorCoordinationPlugin {
    fn name(&self) -> &'static str {
        "mediator_coordination"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|_| {
            tracing::error!("STORAGE_DIRPATH env variable required");
            PluginError::InitError
        })?;

        // Expect DID document from file system
        if did_endpoint::validate_diddoc(&storage_dirpath).is_err() {
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
        let storage_dirpath = std::env::var("STORAGE_DIRPATH").expect(msg);

        let mut fs = StdFileSystem;
        let diddoc = util::read_diddoc(&fs, &storage_dirpath).expect(msg);
        let keystore = util::read_keystore(&mut fs, &storage_dirpath).expect(msg);

        web::routes(diddoc, keystore)
    }
}
