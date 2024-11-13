pub(crate) mod coord;
pub mod error;
pub(crate) mod handler;
pub(crate) mod midlw;

pub use self::midlw::{pack_response_message, unpack_didcomm_message};

use axum::{middleware, routing::post, Router};
use database::Repository;
use did_utils::{didcore::Document, jwk::Jwk};
use keystore::KeyStore;
use std::{string, sync::Arc};

use crate::{
    didcomm::bridge::{LocalDIDResolver, LocalSecretsResolver},
    model::stateful::entity::{Connection, RoutedMessage, Secrets},
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
        .with_state(state)
}

#[derive(Clone)]
pub struct AppState {
    // Metadata
    pub public_domain: String,

    // Crypto identity
    pub diddoc: Document,
    pub assertion_jwk: (String, Jwk),

    // DIDComm Resolvers
    pub(crate) did_resolver: LocalDIDResolver,
    pub(crate) secrets_resolver: LocalSecretsResolver,

    // Persistence layer
    pub(crate) repository: Option<AppStateRepository>,

    // Disclosed protocols as strings e.g `coordinate-mediation/2.0`
    pub(crate) supported_protocols: Option<Vec<String>>,
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
        keystore: KeyStore,
        disclosed_protocols: Option<Vec<String>>,
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
            supported_protocols: disclosed_protocols,
        }
    }
}
