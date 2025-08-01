use crate::{manager::MessagePluginContainer, web};
use axum::Router;
use database::get_or_init_database;
use database::Repository;
use did_endpoint::persistence::DidDocumentRepository;
use did_utils::didcore::Document as DidDocument;
use keystore::Keystore;
use mongodb::{bson::doc, Database};
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
    diddoc: Option<DidDocument>,
    msg_types: Option<Vec<String>>,
    keystore: Option<Keystore>,
}

struct DidcommMessagingPluginEnv {
    public_domain: String,
}

/// Loads environment variables required for this plugin
fn load_plugin_env() -> Result<DidcommMessagingPluginEnv, PluginError> {
    let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").map_err(|err| {
        PluginError::InitError(format!(
            "SERVER_PUBLIC_DOMAIN env variable required: {err:?}"
        ))
    })?;

    Ok(DidcommMessagingPluginEnv { public_domain })
}

impl Plugin for DidcommMessaging {
    fn name(&self) -> &'static str {
        "didcomm_messaging"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        use aws_config::BehaviorVersion;
        use tokio::{runtime::Handle, task};

        let env = load_plugin_env()?;

        let db = get_or_init_database();
        let repository = DidDocumentRepository::from_db(&db);
        let keystore = task::block_in_place(move || {
            Handle::current().block_on(async move {
                let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
                Keystore::with_aws_secrets_manager(&aws_config).await
            })
        });

        // Expect DID document from the repository
        if let Err(err) = did_endpoint::validate_diddoc(&keystore, &repository) {
            return Err(PluginError::InitError(format!(
                "DID document validation failed: {err:?}"
            )));
        }

        let diddoc = task::block_in_place(move || {
            Handle::current().block_on(async move {
                repository
                    .find_one_by(doc! {})
                    .await
                    .map_err(|e| PluginError::Other(e.to_string()))?
                    .ok_or_else(|| {
                        PluginError::Other("Missing did.json from repository".to_string())
                    })
            })
        })?;
        let diddoc = diddoc.diddoc;

        // Load message container
        let mut container = MessagePluginContainer::new();
        if let Err(err) = container.load() {
            return Err(PluginError::InitError(format!(
                "Error loading didcomm messages container: {err:?}"
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

        // Save the environment,MongoDB connection and didcomm message types in the struct
        self.env = Some(env);
        self.db = Some(db);
        self.diddoc = Some(diddoc);
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
        let diddoc = self.diddoc.as_ref().ok_or(PluginError::Other(
            "Failed to get diddoc. Check if the plugin is mounted".to_owned(),
        ))?;
        let keystore = self.keystore.as_ref().ok_or(PluginError::Other(
            "Failed to get keystore. Check if the plugin is mounted".to_owned(),
        ))?;

        // Load persistence layer
        let repository = AppStateRepository {
            connection_repository: Arc::new(MongoConnectionRepository::from_db(db)),
            keystore: keystore.clone(),
            message_repository: Arc::new(MongoMessagesRepository::from_db(db)),
        };

        // Initialize circuit breaker
        let db_breaker = CircuitBreaker::new()
            .retries(3)
            .half_open_max_failures(3)
            .reset_timeout(Duration::from_secs(30))
            .exponential_backoff(Duration::from_millis(50));

        // Compile state
        let state = AppState::from(
            env.public_domain.clone(),
            diddoc.clone(),
            Some(msg_types.clone()),
            Some(repository),
            db_breaker,
        )
        .map_err(|err| PluginError::Other(format!("Failed to load app state: {err:?}")))?;

        // Build router
        Ok(web::routes(Arc::new(state)))
    }
}
