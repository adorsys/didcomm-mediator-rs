use super::{didgen, persistence::*, web};
use axum::Router;
use database::{get_or_init_database, Repository};
use keystore::Keystore;
use mongodb::Database;
use plugin_api::{Plugin, PluginError};
use std::sync::Arc;

#[derive(Default)]
pub struct DidEndpoint {
    env: Option<DidEndpointEnv>,
    state: Option<DidEndPointState>,
    db: Option<Database>,
}

struct DidEndpointEnv {
    server_public_domain: String,
}

#[derive(Clone)]
pub(crate) struct DidEndPointState {
    pub(crate) keystore: Keystore,
    pub(crate) repository: Arc<dyn Repository<MediatorDidDocument>>,
}

fn get_env() -> Result<DidEndpointEnv, PluginError> {
    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
        PluginError::InitError("SERVER_PUBLIC_DOMAIN env variable required".to_owned())
    })?;

    Ok(DidEndpointEnv {
        server_public_domain,
    })
}

impl Plugin for DidEndpoint {
    fn name(&self) -> &'static str {
        "did_endpoint"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        let env = get_env()?;

        self.db = Some(get_or_init_database());

        let repository = DidDocumentRepository::from_db(self.db.as_ref().unwrap());
        let keystore = Keystore::with_mongodb();

        if didgen::validate_diddoc(&keystore, &repository).is_err() {
            tracing::debug!("diddoc validation failed, will generate one");

            didgen::didgen(&env.server_public_domain, &keystore, &repository).map_err(|_| {
                PluginError::InitError(
                    "failed to generate an initial keystore and its DID document".to_owned(),
                )
            })?;
        };

        self.env = Some(env);
        self.state = Some(DidEndPointState {
            keystore,
            repository: Arc::new(repository),
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
