use database::Repository;
use did_utils::didcore::Document;
use std::sync::Arc;

use crate::{
    repository::entity::{Connection, RoutedMessage, Secrets},
    resolvers::{LocalDIDResolver, LocalSecretsResolver},
};

#[derive(Clone)]
pub struct AppState {
    // Metadata
    pub public_domain: String,

    // Crypto identity
    pub diddoc: Document,

    // DIDComm Resolvers
    pub did_resolver: LocalDIDResolver,
    pub secrets_resolver: LocalSecretsResolver,

    // Persistence layer
    pub repository: Option<AppStateRepository>,
}

#[derive(Clone)]
pub struct AppStateRepository {
    pub connection_repository: Arc<dyn Repository<Connection>>,
    pub secret_repository: Arc<dyn Repository<Secrets>>,
    pub message_repository: Arc<dyn Repository<RoutedMessage>>,
}

impl AppState {
    pub fn from(
        public_domain: String,
        diddoc: Document,
        repository: Option<AppStateRepository>,
    ) -> Self {
        let did_resolver = LocalDIDResolver::new(&diddoc);
        let secrets_resolver = {
            let repository = repository.as_ref().expect("Missing persistence layer");
            LocalSecretsResolver::new(repository.secret_repository.clone())
        };

        Self {
            public_domain,
            diddoc,
            did_resolver,
            secrets_resolver,
            repository,
        }
    }
}
