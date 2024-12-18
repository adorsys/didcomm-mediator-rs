use std::sync::{Arc, Mutex};

use super::{
    models::{retrieve_or_generate_oob_inv, retrieve_or_generate_qr_image},
    web,
};
use axum::Router;
use filesystem::{FileSystem, StdFileSystem};
use plugin_api::{Plugin, PluginError};

#[derive(Default)]
pub struct OOBMessages {
    env: Option<OOBMessagesEnv>,
    state: Option<OOBMessagesState>,
}

struct OOBMessagesEnv {
    storage_dirpath: String,
    server_public_domain: String,
}

#[derive(Clone)]
pub(crate) struct OOBMessagesState {
    pub(crate) filesystem: Arc<Mutex<dyn FileSystem>>,
    pub(crate) oobmessage: String,
}

fn get_env() -> Result<OOBMessagesEnv, PluginError> {
    let storage_dirpath = std::env::var("STORAGE_DIRPATH")
        .map_err(|_| PluginError::InitError("STORAGE_DIRPATH env variable required".to_owned()))?;

    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
        PluginError::InitError("SERVER_PUBLIC_DOMAIN env variable required".to_owned())
    })?;

    Ok(OOBMessagesEnv {
        storage_dirpath,
        server_public_domain,
    })
}

impl Plugin for OOBMessages {
    fn name(&self) -> &'static str {
        "oob_messages"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        let env = get_env()?;
        let mut fs = StdFileSystem;

        let oob_inv =
            retrieve_or_generate_oob_inv(&mut fs, &env.server_public_domain, &env.storage_dirpath)
                .map_err(|err| {
                    PluginError::InitError(format!(
                        "Error retrieving or generating OOB invitation: {err}"
                    ))
                })?;

        tracing::debug!("Out Of Band Invitation: {}", oob_inv);

        retrieve_or_generate_qr_image(&mut fs, &env.storage_dirpath, &oob_inv).map_err(|err| {
            PluginError::InitError(format!(
                "Error retrieving or generating QR code image: {err}"
            ))
        })?;

        self.env = Some(env);
        let oobmessage: Vec<&str> = oob_inv.split("/_").collect();
        let oobmessage = oobmessage.get(1).unwrap_or(&"").to_string();

        self.state = Some(OOBMessagesState {
            filesystem: Arc::new(Mutex::new(fs)),
            oobmessage,
        });

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Result<Router, PluginError> {
        let state = self.state.as_ref().ok_or(PluginError::Other(
            "missing state, plugin not mounted".to_owned(),
        ))?;
        Ok(web::routes(Arc::new(state.clone())))
    }
}
