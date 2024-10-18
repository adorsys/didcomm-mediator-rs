use database::Repository;
use did_utils::didcore::Document;
use keystore::Secrets;
use std::sync::Arc;

use crate::{
    repository::entity::{Connection, RoutedMessage},
    utils::resolvers::{LocalDIDResolver, LocalSecretsResolver},
};

#[derive(Clone)]
pub struct AppState {
    // Metadata
    pub public_domain: String,

    // Crypto identity
    pub diddoc: Document,

    // KeyStore
    pub keystore: Arc<dyn Repository<Secrets>>,

    // DIDComm Resolvers
    pub did_resolver: LocalDIDResolver,
    pub secrets_resolver: LocalSecretsResolver,

    // Persistence layer
    pub repository: Option<AppStateRepository>,
}

#[derive(Clone)]
pub struct AppStateRepository {
    pub connection_repository: Arc<dyn Repository<Connection>>,
    pub message_repository: Arc<dyn Repository<RoutedMessage>>,
}

impl AppState {
    pub fn from(
        public_domain: String,
        diddoc: Document,
        keystore: Arc<dyn Repository<Secrets>>,
        repository: Option<AppStateRepository>,
    ) -> Self {
        let did_resolver = LocalDIDResolver::new(&diddoc);
        let secrets_resolver = LocalSecretsResolver::new(keystore.clone());

        Self {
            public_domain,
            diddoc,
            keystore,
            did_resolver,
            secrets_resolver,
            repository,
        }
    }
}
