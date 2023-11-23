mod coord;

use axum::Router;
use did_endpoint::util::keystore::KeyStore;
use did_utils::didcore::Document;

// #[derive(Clone)]
// pub struct AppState {
//     did_web_endpoint: String,
//     diddoc: Document,
//     assertion_jwk: (String, Jwk),
//     did_resolver: LocalDIDResolver,
//     secrets_resolver: LocalSecretsResolver,
// }

pub fn routes(_diddoc: Document, _keystore: KeyStore) -> Router {
    Router::new()
}
