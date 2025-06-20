use super::{didgen, web};
use aws_config::BehaviorVersion;
use axum::Router;
use filesystem::FileSystem;
use keystore::Keystore;
use plugin_api::{Plugin, PluginError};
use std::sync::{Arc, Mutex};
use tokio::{runtime::Handle, task};

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
    pub(crate) keystore: Keystore,
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
        let mut filesystem = filesystem::StdFileSystem;
        let keystore = task::block_in_place(move || {
            Handle::current().block_on(async move {
                let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
                Keystore::with_aws_secrets_manager(&aws_config).await
            })
        });

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
                PluginError::InitError(
                    "failed to generate an initial keystore and its DID document".to_owned(),
                )
            })?;
        };

        self.env = Some(env);
        self.state = Some(DidEndPointState {
            keystore,
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
