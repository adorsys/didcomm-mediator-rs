use axum::Router;
use database::Repository;
use keystore::filesystem::StdFileSystem;
use mongodb::{options::ClientOptions, Client, Database};
use plugin_api::{Plugin, PluginError};
use std::sync::Arc;

use crate::{
    repository::stateful::{
        MongoConnectionRepository, MongoMessagesRepository, MongoSecretsRepository,
    },
    util,
    web::{self, AppState, AppStateRepository},
};
#[derive(Default)]
pub struct MediatorCoordinationPlugin {
    env: Option<MediatorCoordinationPluginEnv>,
    db: Option<Database>,
}

struct MediatorCoordinationPluginEnv {
    public_domain: String,
    storage_dirpath: String,
    mongo_uri: String,
    mongo_dbn: String,
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

    let mongo_uri = std::env::var("MONGO_URI").map_err(|_| {
        tracing::error!("MONGO_URI env variable required");
        PluginError::InitError
    })?;

    let mongo_dbn = std::env::var("MONGO_DBN").map_err(|_| {
        tracing::error!("MONGO_DBN env variable required");
        PluginError::InitError
    })?;

    Ok(MediatorCoordinationPluginEnv {
        public_domain,
        storage_dirpath,
        mongo_uri,
        mongo_dbn,
    })
}

impl Plugin for MediatorCoordinationPlugin {
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
        let db = load_mongo_connector(&env.mongo_uri, &env.mongo_dbn)?;

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
        let mut fs = StdFileSystem;
        let diddoc = util::read_diddoc(&fs, &env.storage_dirpath).expect(msg);
        let secret_repository = Arc::new(MongoSecretsRepository::from_db(&db));

        // Fetch the necessary secrets for routing from the database
        let keystore = secret_repository.get_collection().expect("Failed to retrieve secrets");
        
        // Load persistence layer
        let repository = AppStateRepository {
            connection_repository: Arc::new(MongoConnectionRepository::from_db(&db)),
            secret_repository: Arc::new(MongoSecretsRepository::from_db(&db)),
            message_repository: Arc::new(MongoMessagesRepository::from_db(&db)),
        };

        // Compile state
        let state = AppState::from(
            env.public_domain.clone(),
            diddoc,
            keystore,
            Some(repository),
        );

        // Build router
        web::routes(Arc::new(state))
    }
}

fn load_mongo_connector(mongo_uri: &str, mongo_dbn: &str) -> Result<Database, PluginError> {
    let task = async {
        // Parse a connection string into an options struct.
        let client_options = ClientOptions::parse(mongo_uri).await.map_err(|_| {
            tracing::error!("Failed to parse Mongo URI");
            PluginError::InitError
        })?;

        // Get a handle to the deployment.
        let client = Client::with_options(client_options).map_err(|_| {
            tracing::error!("Failed to create MongoDB client");
            PluginError::InitError
        })?;

        // Get a handle to a database.
        Ok(client.database(mongo_dbn))
    };

    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(task))
}
