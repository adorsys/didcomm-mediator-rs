use super::store::{InMemoryStore, Store};
use axum::Router;
use did_endpoint::persistence::DidDocumentRepository;
use did_utils::didcore::Document;
use mongodb::bson::doc;
use plugin_api::{Plugin, PluginError};
use std::sync::{Arc, Mutex};
use tokio::{runtime::Handle, task};

use super::{
    models::{retrieve_or_generate_oob_inv, retrieve_or_generate_qr_image},
    web,
};
use database::Repository;

#[derive(Default)]
pub struct OOBMessages {
    env: Option<OOBMessagesEnv>,
    state: Option<OOBMessagesState>,
}

struct OOBMessagesEnv {
    server_public_domain: String,
}

#[derive(Clone)]
pub(crate) struct OOBMessagesState {
    pub(crate) store: Arc<Mutex<dyn Store>>,
    pub(crate) diddoc: Document,
    pub(crate) server_public_domain: String,
}

fn get_env() -> Result<OOBMessagesEnv, PluginError> {
    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
        PluginError::InitError("SERVER_PUBLIC_DOMAIN env variable required".to_owned())
    })?;

    Ok(OOBMessagesEnv {
        server_public_domain,
    })
}

impl Plugin for OOBMessages {
    fn name(&self) -> &'static str {
        "oob_messages"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        let env = get_env()?;
        let mut store = InMemoryStore::default();

        let db = database::get_or_init_database();
        let repository = DidDocumentRepository::from_db(&db);

        let diddoc = task::block_in_place(move || {
            Handle::current().block_on(async move {
                repository
                    .find_one_by(doc! {})
                    .await
                    .map_err(|e| PluginError::Other(e.to_string()))?
                    .ok_or_else(|| PluginError::Other("Missing did.json from repository".to_string()))
            })
        })?;
        let diddoc = diddoc.diddoc;

        let oob_inv = retrieve_or_generate_oob_inv(
            &mut store,
            &diddoc,
            &env.server_public_domain,
        )
        .map_err(|err| {
            PluginError::InitError(format!(
                "Error retrieving or generating OOB invitation: {err}"
            ))
        })?;

        tracing::debug!("Out Of Band Invitation: {}", oob_inv);

        retrieve_or_generate_qr_image(&mut store, &oob_inv).map_err(|err| {
            PluginError::InitError(format!(
                "Error retrieving or generating QR code image: {err}"
            ))
        })?;

        let server_public_domain = env.server_public_domain.clone();
        self.env = Some(env);
        self.state = Some(OOBMessagesState {
            store: Arc::new(Mutex::new(store)),
            diddoc,
            server_public_domain,
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
