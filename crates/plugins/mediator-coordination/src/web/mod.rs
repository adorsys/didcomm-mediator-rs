use axum::Router;
use did_utils::didcore::Document;
use keystore::KeyStore;

pub(crate) fn routes(_diddoc: Document, _keystore: KeyStore) -> Router {
    Router::new()
}
