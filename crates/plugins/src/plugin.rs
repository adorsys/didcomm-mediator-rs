use crate::web;
use axum::Router;
use filesystem::StdFileSystem;
use mongodb::Database;
use plugin_api::{Plugin, PluginError};
use shared::{
    repository::{MongoConnectionRepository, MongoMessagesRepository},
    state::{AppState, AppStateRepository},
    utils,
};
use std::sync::Arc;

#[derive(Default)]
pub struct MediatorCoordination {
    env: Option<MediatorCoordinationPluginEnv>,
    db: Option<Database>,
}

struct MediatorCoordinationPluginEnv {
    public_domain: String,
    storage_dirpath: String,
}

/// Loads environment variables required for this plugin
fn load_plugin_env() -> Result<MediatorCoordinationPluginEnv, PluginError> {
    let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
        tracing::error!("SERVER_PUBLIC_DOMAIN env variable required");
        PluginError::InitError
    })?;

    let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|_| {
        tracing::error!("STORAGE_DIRPATH env variable required");
        PluginError::InitError
    })?;

    Ok(MediatorCoordinationPluginEnv {
        public_domain,
        storage_dirpath,
    })
}

impl Plugin for MediatorCoordination {
    fn name(&self) -> &'static str {
        "mediator_coordination"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        let env = load_plugin_env()?;

        // Expect DID document from file system
        if did_endpoint::validate_diddoc(&env.storage_dirpath).is_err() {
            tracing::error!("diddoc validation failed; is plugin did-endpoint mounted?");
            return Err(PluginError::InitError);
        }

        // Check connectivity to database
        let db = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let db_instance = database::get_or_init_database();
                let db_lock = db_instance.lock().await;
                db_lock.clone()
            })
        });

        // Save the environment and MongoDB connection in the struct
        self.env = Some(env);
        self.db = Some(db);

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        // Ensure the plugin is properly mounted
        let env = self.env.as_ref().expect("Plugin not mounted");
        let db = self.db.as_ref().expect("Plugin not mounted");

        let msg = "This should not occur following successful mounting.";

        // Load crypto identity
        let fs = StdFileSystem;
        let diddoc = utils::read_diddoc(&fs, &env.storage_dirpath).expect(msg);

        // Load persistence layer
        let repository = AppStateRepository {
            connection_repository: Arc::new(MongoConnectionRepository::from_db(&db)),
            keystore: Arc::new(keystore::KeyStore::get()),
            message_repository: Arc::new(MongoMessagesRepository::from_db(&db)),
        };

        // Compile state
        let state = AppState::from(env.public_domain.clone(), diddoc, Some(repository));

        // Build router
        web::routes(Arc::new(state))
    }
}
