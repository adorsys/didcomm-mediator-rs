use axum::Router;
use did_endpoint::{didgen, util::filesystem::StdFileSystem};
use mongodb::{error::Error as MongoError, options::ClientOptions, Client, Database};
use server_plugin::{Plugin, PluginError};
use std::sync::Arc;

use crate::{
    repository::stateful::coord::MongoConnectionRepository,
    util,
    web::{self, AppState, AppStateRepository},
};

#[derive(Default)]
pub struct MediatorCoordinationPlugin;

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

    fn mount(&self) -> Result<(), PluginError> {
        // Load environment variables required for this plugin

        let MediatorCoordinationPluginEnv {
            storage_dirpath,
            mongo_uri,
            mongo_dbn,
            ..
        } = load_plugin_env()?;

        // Expect DID document from file system

        if didgen::validate_diddoc(&storage_dirpath).is_err() {
            tracing::error!("diddoc validation failed; is plugin did-endpoint mounted?");
            return Err(PluginError::InitError);
        }

        // Check connectivity to database

        if load_mongo_connector(&mongo_uri, &mongo_dbn).is_err() {
            tracing::error!("could not establish connectivity with mongodb");
            return Err(PluginError::InitError);
        }

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        let msg = "This should not occur following successful mounting.";
        let MediatorCoordinationPluginEnv {
            public_domain,
            storage_dirpath,
            mongo_uri,
            mongo_dbn,
        } = load_plugin_env().expect(msg);

        // Load crypto identity
        let mut fs = StdFileSystem;
        let diddoc = util::read_diddoc(&fs, &storage_dirpath).expect(msg);
        let keystore = util::read_keystore(&mut fs, &storage_dirpath).expect(msg);

        // Load connection to database
        let db = load_mongo_connector(&mongo_uri, &mongo_dbn).expect(msg);

        // Load persistence layer
        let repository = AppStateRepository {
            connection_repository: Arc::new(MongoConnectionRepository::from_db(&db)),
        };

        // Compile state
        let state = AppState::from(public_domain, diddoc, keystore, Some(repository));

        // Build router
        web::routes(Arc::new(state))
    }
}

fn load_mongo_connector(mongo_uri: &str, mongo_dbn: &str) -> Result<Database, MongoError> {
    let task = async {
        // Parse a connection string into an options struct.
        let client_options = ClientOptions::parse(mongo_uri).await?;

        // Get a handle to the deployment.
        let client = Client::with_options(client_options)?;

        // Get a handle to a database.
        Ok(client.database(mongo_dbn))
    };

    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(task))
}
