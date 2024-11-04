use super::{didgen, web};
use axum::Router;
use database::Repository;
use keystore::Secrets;
use plugin_api::{Plugin, PluginError};
use std::sync::{Arc, Mutex};
use filesystem::FileSystem;

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
    pub(crate) keystore: Arc<dyn Repository<Secrets>>,
    pub(crate) filesystem: Arc<Mutex<dyn FileSystem>>,
}

fn get_env() -> Result<DidEndpointEnv, PluginError> {
    let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|_| {
        tracing::error!("STORAGE_DIRPATH env variable required");
        PluginError::InitError
    })?;

    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
        tracing::error!("SERVER_PUBLIC_DOMAIN env variable required");
        PluginError::InitError
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
        let mut filesystem = filesystem::StdFileSystem;
        let keystore = keystore::KeyStore::get();

        if didgen::validate_diddoc(env.storage_dirpath.as_ref(), &keystore, &mut filesystem)
            .is_err()
        {
            tracing::debug!("diddoc validation failed, will generate one");

            didgen::didgen(
                env.storage_dirpath.as_ref(),
                &env.server_public_domain,
                &keystore,
                &mut filesystem,
            )
            .map_err(|_| {
                tracing::error!("failed to generate an initial keystore and its DID document");
                PluginError::InitError
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

    fn routes(&self) -> Router {
        let state = self.state.as_ref().expect("Plugin not mounted");
        web::routes(Arc::new(state.clone()))
    }
}