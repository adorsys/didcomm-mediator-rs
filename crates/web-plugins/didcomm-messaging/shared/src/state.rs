use database::Repository;
use did_utils::didcore::Document;
use keystore::Keystore;
use std::sync::Arc;

use crate::{
    breaker::CircuitBreaker,
    repository::entity::{Connection, RoutedMessage},
    utils::resolvers::{LocalDIDResolver, LocalSecretsResolver},
};

#[derive(Clone)]
pub struct AppState {
    // Server public domain
    pub public_domain: String,
    // DID Document
    pub diddoc: Document,
    // DIDComm Resolvers
    pub did_resolver: LocalDIDResolver,
    pub secrets_resolver: LocalSecretsResolver,
    // Persistence layer
    pub repository: Option<AppStateRepository>,
    // Circuit breaker
    pub db_circuit_breaker: CircuitBreaker,
    // disclosed protocols
    pub disclose_protocols: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct AppStateRepository {
    pub connection_repository: Arc<dyn Repository<Connection>>,
    pub message_repository: Arc<dyn Repository<RoutedMessage>>,
    pub keystore: Keystore,
}

impl AppState {
    pub fn from(
        public_domain: String,
        diddoc: Document,
        disclose_protocols: Option<Vec<String>>,
        repository: Option<AppStateRepository>,
        db_circuit_breaker: CircuitBreaker,
    ) -> eyre::Result<Self> {
        let did_resolver = LocalDIDResolver::new(&diddoc);
        let keystore = repository
            .as_ref()
            .ok_or_else(|| eyre::eyre!("Missing persistence layer"))?
            .keystore
            .clone();

        let secrets_resolver = LocalSecretsResolver::new(keystore);

        Ok(Self {
            public_domain,
            diddoc,
            did_resolver,
            secrets_resolver,
            repository,
            db_circuit_breaker,
            disclose_protocols,
        })
    }
}
