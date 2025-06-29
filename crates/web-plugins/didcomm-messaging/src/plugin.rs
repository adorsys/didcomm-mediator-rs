use crate::{manager::MessagePluginContainer, web};
use axum::Router;
use filesystem::StdFileSystem;
use keystore::Keystore;
use mongodb::Database;
use once_cell::sync::OnceCell;
use plugin_api::{Plugin, PluginError};
use shared::{
    breaker::CircuitBreaker,
    repository::{MongoConnectionRepository, MongoMessagesRepository},
    state::{AppState, AppStateRepository},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

pub(crate) static MESSAGE_CONTAINER: OnceCell<RwLock<MessagePluginContainer>> = OnceCell::new();

#[derive(Default)]
pub struct DidcommMessaging {
    env: Option<DidcommMessagingPluginEnv>,
    db: Option<Database>,
    msg_types: Option<Vec<String>>,
    keystore: Option<Keystore>,
}

struct DidcommMessagingPluginEnv {
    public_domain: String,
    storage_dirpath: String,
}

/// Loads environment variables required for this plugin
fn load_plugin_env() -> Result<DidcommMessagingPluginEnv, PluginError> {
    let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|err| {
        PluginError::InitError(format!(
            "SERVER_PUBLIC_DOMAIN env variable required: {:?}",
            err
        ))
    })?;

    let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|err| {
        PluginError::InitError(format!("STORAGE_DIRPATH env variable required: {:?}", err))
    })?;

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
        use aws_config::BehaviorVersion;
        use tokio::{runtime::Handle, task};

        let env = load_plugin_env()?;

        let mut filesystem = filesystem::StdFileSystem;
        let keystore = task::block_in_place(move || {
            Handle::current().block_on(async move {
                let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
                Keystore::with_aws_secrets_manager(&aws_config).await
            })
        });

        // Expect DID document from file system
        if let Err(err) =
            did_endpoint::validate_diddoc(env.storage_dirpath.as_ref(), &keystore, &mut filesystem)
        {
            return Err(PluginError::InitError(format!(
                "DID document validation failed: {:?}",
                err
            )));
        }

        // Load message container
        let mut container = MessagePluginContainer::new();
        if let Err(err) = container.load() {
            return Err(PluginError::InitError(format!(
                "Error loading didcomm messages container: {:?}",
                err
            )));
        }

        // Get didcomm message types
        let msg_types = container
            .didcomm_routes()
            .map_err(|_| PluginError::InitError("Failed to get didcomm message types".to_owned()))?
            .messages_types();

        // Set the message container
        MESSAGE_CONTAINER
            .set(RwLock::new(container))
            .map_err(|_| PluginError::InitError("Container already initialized".to_owned()))?;

        // Check connectivity to database
        let db = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let db_instance = database::get_or_init_database();
                db_instance.clone()
            })
        });

        // Save the environment,MongoDB connection and didcomm message types in the struct
        self.env = Some(env);
        self.db = Some(db);
        self.msg_types = Some(msg_types);
        self.keystore = Some(keystore);

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
        let msg_types = self.msg_types.as_ref().ok_or(PluginError::Other(
            "Failed to get message types. Check if the plugin is mounted".to_owned(),
        ))?;
        let keystore = self.keystore.as_ref().ok_or(PluginError::Other(
            "Failed to get keystore. Check if the plugin is mounted".to_owned(),
        ))?;

        // Load crypto identity
        let fs = StdFileSystem;
        let diddoc = shared::utils::read_diddoc(&fs, &env.storage_dirpath).map_err(|err| {
            PluginError::Other(format!(
                "This should not occur following successful mounting: {:?}",
                err
            ))
        })?;
        // Load persistence layer
        let repository = AppStateRepository {
            connection_repository: Arc::new(MongoConnectionRepository::from_db(db)),
            keystore: keystore.clone(),
            message_repository: Arc::new(MongoMessagesRepository::from_db(db)),
        };

        // Initialize circuit breakers
        let db_breaker = CircuitBreaker::new()
            .retries(3)
            .half_open_max_failures(3)
            .reset_timeout(Duration::from_secs(30))
            .exponential_backoff(Duration::from_millis(50));

        // Compile state
        let state = AppState::from(
            env.public_domain.clone(),
            diddoc,
            Some(msg_types.clone()),
            Some(repository),
            db_breaker,
        )
        .map_err(|err| PluginError::Other(format!("Failed to load app state: {:?}", err)))?;

        // Build router
        Ok(web::routes(Arc::new(state)))
    }
}
