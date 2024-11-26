use crate::{manager::MessagePluginContainer, web};
use axum::Router;
use filesystem::StdFileSystem;
use mongodb::Database;
use once_cell::sync::OnceCell;
use plugin_api::{Plugin, PluginError};
use shared::{
    repository::{MongoConnectionRepository, MongoMessagesRepository},
    state::{AppState, AppStateRepository},
    utils,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) static MESSAGE_CONTAINER: OnceCell<RwLock<MessagePluginContainer>> = OnceCell::new();

#[derive(Default)]
pub struct DidcommMessaging {
    env: Option<DidcommMessagingPluginEnv>,
    db: Option<Database>,
}

struct DidcommMessagingPluginEnv {
    public_domain: String,
    storage_dirpath: String,
}

/// Loads environment variables required for this plugin
fn load_plugin_env() -> Result<DidcommMessagingPluginEnv, PluginError> {
    let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|_| {
        PluginError::InitError("SERVER_PUBLIC_DOMAIN env variable required".to_owned())
    })?;

    let storage_dirpath = std::env::var("STORAGE_DIRPATH")
        .map_err(|_| PluginError::InitError("STORAGE_DIRPATH env variable required".to_owned()))?;

    Ok(DidcommMessagingPluginEnv {
        public_domain,
        storage_dirpath,
    })
}

impl Plugin for DidcommMessaging {
    fn name(&self) -> &'static str {
        "didcomm_messaging"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        let env = load_plugin_env()?;

        let mut filesystem = filesystem::StdFileSystem;
        let keystore = keystore::KeyStore::get();

        // Expect DID document from file system
        if did_endpoint::validate_diddoc(env.storage_dirpath.as_ref(), &keystore, &mut filesystem)
            .is_err()
        {
            return Err(PluginError::InitError(
                "diddoc validation failed; is plugin did-endpoint mounted?".to_owned(),
            ));
        }

        // Load message container
        let mut container = MessagePluginContainer::new();
        if let Err(err) = container.load() {
            tracing::error!("Error loading message container: {:?}", err);
            return Err(PluginError::InitError);
        }

        MESSAGE_CONTAINER.set(RwLock::new(container)).map_err(|_| {
            tracing::error!("Container already initialized");
            PluginError::InitError
        })?;

        // Check connectivity to database
        let db = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let db_instance = database::get_or_init_database();
                let db_lock = db_instance.read().await;
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

    fn routes(&self) -> Result<Router, PluginError> {
        // Ensure the plugin is properly mounted
        let env = self.env.as_ref().ok_or(PluginError::Other(
            "Failed to get environment variables. Check if the plugin is mounted".to_owned(),
        ))?;
        let db = self.db.as_ref().ok_or(PluginError::Other(
            "Failed to get database handle. Check if the plugin is mounted".to_owned(),
        ))?;

        // Load crypto identity
        let fs = StdFileSystem;
        let diddoc = utils::read_diddoc(&fs, &env.storage_dirpath).map_err(|_| {
            PluginError::Other("This should not occur following successful mounting.".to_owned())
        })?;

        // Load persistence layer
        let repository = AppStateRepository {
            connection_repository: Arc::new(MongoConnectionRepository::from_db(&db)),
            keystore: Arc::new(keystore::KeyStore::get()),
            message_repository: Arc::new(MongoMessagesRepository::from_db(&db)),
        };

        // Compile state
        let state = AppState::from(env.public_domain.clone(), diddoc, None, Some(repository))
            .map_err(|err| {
                tracing::error!("Failed to load app state: {:?}", err);
                PluginError::Other("Failed to load app state".to_owned())
            })?;

        // Build router
        Ok(web::routes(Arc::new(state)))
    }
}
