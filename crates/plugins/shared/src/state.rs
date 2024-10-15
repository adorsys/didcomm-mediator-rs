use database::Repository;
use did_utils::{didcore::Document, jwk::Jwk};
use std::sync::Arc;

use crate::{
    repository::entity::{Connection, RoutedMessage, Secrets},
    resolvers::{LocalDIDResolver, LocalSecretsResolver},
    util,
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
        let (did_url, assertion_pubkey) = util::extract_assertion_key(&diddoc)
            .expect("Failed to retrieve assertion key details from server DID document");
        let assertion_jwk = (
            did_url,
            keystore
                .find_keypair(&assertion_pubkey)
                .expect("Unsuccessful keystore search"),
        );

        let did_resolver = LocalDIDResolver::new(&diddoc);
        let secrets_resolver = {
            let (vm_id, jwk) = util::extract_agreement_key(&diddoc)
                .expect("Failed to retrieve agreement key details from server DID document");
            let secret = keystore
                .find_keypair(&jwk)
                .expect("Unsuccessful keystore search");
            LocalSecretsResolver::new(&vm_id, &secret)
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
