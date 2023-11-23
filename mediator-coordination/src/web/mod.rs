mod coord;

use axum::{routing::post, Router};
use did_endpoint::util::keystore::KeyStore;
use did_utils::{didcore::Document, key_jwk::jwk::Jwk};

use crate::{
    didcomm::bridge::{LocalDIDResolver, LocalSecretsResolver},
    util,
};

pub fn routes(public_domain: String, diddoc: Document, keystore: KeyStore) -> Router {
    let state = AppState::from(public_domain, diddoc, keystore);

    Router::new()
        .route(
            "/mediate",
            post(coord::handler::process_didcomm_mediation_request_message),
        )
        .with_state(state)
}

#[derive(Clone)]
#[allow(unused)]
pub struct AppState {
    public_domain: String,

    diddoc: Document,
    assertion_jwk: (String, Jwk),

    did_resolver: LocalDIDResolver,
    secrets_resolver: LocalSecretsResolver,
}

impl AppState {
    pub fn from(public_domain: String, diddoc: Document, keystore: KeyStore) -> Self {
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
        }
    }
}
