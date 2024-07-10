use axum::Router;
use did_endpoint::util::keystore::KeyStore;
use did_utils::didcore::Document;

pub(crate) fn routes(_diddoc: Document, _keystore: KeyStore) -> Router {
    Router::new()
}
