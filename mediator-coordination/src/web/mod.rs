mod coord;
mod error;
mod handler;
mod midlw;

use axum::{middleware, routing::post, Router};
use did_endpoint::util::keystore::KeyStore;
use did_utils::{didcore::Document, key_jwk::jwk::Jwk};
use std::sync::Arc;

use crate::{
    didcomm::bridge::{LocalDIDResolver, LocalSecretsResolver},
    model::stateful::coord::entity::Connection,
    repository::traits::Repository,
    util,
};

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Unified route for all DIDComm messages
        .route("/", post(handler::process_didcomm_message))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            midlw::unpack_didcomm_message,
        ))
        // Transitive routes
        .route(
            "/mediate",
            post(coord::handler::process_didcomm_mediation_request_message),
        )
        .route(
            "/keylist",
            post(coord::handler::stateful::process_plain_keylist_update_message),
        )
        .with_state(state)
}

#[derive(Clone)]
#[allow(unused)]
pub struct AppState {
    // Metadata
    public_domain: String,

    // Crypto identity
    diddoc: Document,
    assertion_jwk: (String, Jwk),

    // DIDComm Resolvers
    did_resolver: LocalDIDResolver,
    secrets_resolver: LocalSecretsResolver,

    // Persistence layer
    repository: Option<AppStateRepository>,
}

#[derive(Clone)]
pub struct AppStateRepository {
    pub connection_repository: Arc<dyn Repository<Connection>>,
}

impl AppState {
    pub fn from(
        public_domain: String,
        diddoc: Document,
        keystore: KeyStore,
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
            assertion_jwk,
            did_resolver,
            secrets_resolver,
            repository,
        }
    }
}
