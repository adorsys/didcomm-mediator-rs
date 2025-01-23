use super::{didgen, web};
use axum::Router;
use filesystem::FileSystem;
use keystore::{SecureRepository, WrapSecret};
use plugin_api::{Plugin, PluginError};
use shared::utils::get_master_key;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct DidEndpoint {
    env: Option<DidEndpointEnv>,
    state: Option<DidEndPointState>,
}

struct DidEndpointEnv {
    storage_dirpath: String,
    server_public_domain: String,
}

#[derive(Clone)]
pub(crate) struct DidEndPointState {
    pub(crate) keystore: Arc<dyn SecureRepository<WrapSecret>>,
    pub(crate) filesystem: Arc<Mutex<dyn FileSystem>>,
}

fn get_env() -> Result<DidEndpointEnv, PluginError> {
    let storage_dirpath = std::env::var("STORAGE_DIRPATH")
        .map_err(|_| PluginError::InitError("STORAGE_DIRPATH env variable required".to_owned()))?;

    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
        PluginError::InitError("SERVER_PUBLIC_DOMAIN env variable required".to_owned())
    })?;

    Ok(DidEndpointEnv {
        storage_dirpath,
        server_public_domain,
    })
}


impl Plugin for DidEndpoint {
    fn name(&self) -> &'static str {
        "did_endpoint"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        let env = get_env()?;
        let master_key = get_master_key()?;

        let mut filesystem = filesystem::StdFileSystem;
        let keystore = keystore::KeyStore::get();

        if didgen::validate_diddoc(
            env.storage_dirpath.as_ref(),
            &keystore,
            &mut filesystem,
            master_key,
        )
        .is_err()
        {
            tracing::debug!("diddoc validation failed, will generate one");

            didgen::didgen(
                env.storage_dirpath.as_ref(),
                &env.server_public_domain,
                &keystore,
                &mut filesystem,
                master_key,
            )
            .map_err(|_| {
                PluginError::InitError(
                    "failed to generate an initial keystore and its DID document".to_owned(),
                )
            })?;
        };

        self.env = Some(env);
        self.state = Some(DidEndPointState {
            keystore: Arc::new(keystore),
            filesystem: Arc::new(Mutex::new(filesystem)),
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
