#![allow(unused)]

use did_endpoint::util::{keystore::KeyStoreError, KeyStore};
use did_utils::{
    didcore::{AssertionMethod, Document, KeyAgreement, KeyFormat, VerificationMethod},
    key_jwk::jwk::Jwk,
};
use serde_json::Error as SerdeError;
use std::io;

/// Custom error type that wraps different kinds of errors that could occur.
#[derive(Debug)]
pub enum DidDocError {
    IoError(io::Error),
    ParseError(SerdeError),
}

/// Converts `io::Error` to `DidDocError::IoError`
impl From<io::Error> for DidDocError {
    fn from(err: io::Error) -> DidDocError {
        DidDocError::IoError(err)
    }
}

/// Converts `SerdeError` to `DidDocError::ParseError`
impl From<SerdeError> for DidDocError {
    fn from(err: SerdeError) -> DidDocError {
        DidDocError::ParseError(err)
    }
}

/// Parse DID document expected to exist on filesystem.
pub fn read_diddoc(storage_dirpath: &str) -> Result<Document, DidDocError> {
    let didpath = format!("{storage_dirpath}/did.json");
    let content = std::fs::read_to_string(didpath)?;
    serde_json::from_str(&content).map_err(Into::into)
}

/// Parse key store expected to exist on filesystem.
pub fn read_keystore(storage_dirpath: &str) -> Result<KeyStore, KeyStoreError> {
    KeyStore::latest(storage_dirpath)
}

/// Generic macro function to look for a key in a DID document.
///
/// if present, return its verification method ID and JWK representation.
macro_rules! extract_key_from_diddoc {
    ($T: ty) => {
        |diddoc: &Document, method: &$T| -> Option<(String, Jwk)> {
            type Rel = $T;

            let id = match method {
                Rel::Reference(reference) => reference,
                Rel::Embedded(vm) => return extract_public_jwk_from_vm(&*vm),
            };

            diddoc.verification_method.as_ref().and_then(|arr| {
                arr.iter()
                    .find(|vm| &vm.id == id)
                    .and_then(extract_public_jwk_from_vm)
            })
        }
    };
}

/// Search an assertion key in a DID document.
///
/// if present, return its verification method ID and JWK representation.
pub fn extract_assertion_key(diddoc: &Document) -> Option<(String, Jwk)> {
    let method = diddoc.assertion_method.as_ref()?.get(0)?;
    extract_key_from_diddoc!(AssertionMethod)(diddoc, method)
}

/// Search an agreement key in a DID document.
///
/// if present, return its verification method ID and JWK representation.
pub fn extract_agreement_key(diddoc: &Document) -> Option<(String, Jwk)> {
    let method = diddoc.key_agreement.as_ref()?.get(0)?;
    extract_key_from_diddoc!(KeyAgreement)(diddoc, method)
}

/// Reads public JWK from verification method.
///
/// Return verification method ID and JWK if present, else Option::None.
fn extract_public_jwk_from_vm(vm: &VerificationMethod) -> Option<(String, Jwk)> {
    vm.public_key.as_ref().and_then(|key| match key {
        KeyFormat::Jwk(jwk) => Some((vm.id.clone(), jwk.clone())),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::Value;

    fn setup() -> String {
        dotenv_flow_read("STORAGE_DIRPATH").unwrap()
    }

    #[test]
    fn can_read_persisted_entities() {
        let storage_dirpath = setup();
        assert!(read_diddoc(&storage_dirpath).is_ok());
        assert!(read_keystore(&storage_dirpath).is_ok());
    }

    #[test]
    fn can_extract_assertion_key() {
        let storage_dirpath = setup();
        let diddoc = read_diddoc(&storage_dirpath).unwrap();

        let (vm_id, jwk) = extract_assertion_key(&diddoc).unwrap();
        let expected_jwk = serde_json::from_str::<Value>(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4"
            }"#,
        )
        .unwrap();

        assert_eq!(vm_id, "did:web:mediators-r-us.com#keys-2");
        assert_eq!(
            json_canon::to_string(&jwk).unwrap(),
            json_canon::to_string(&expected_jwk).unwrap()
        );
    }

    #[test]
    fn can_extract_agreement_key() {
        let storage_dirpath = setup();
        let diddoc = read_diddoc(&storage_dirpath).unwrap();

        let (vm_id, jwk) = extract_agreement_key(&diddoc).unwrap();
        let expected_jwk = serde_json::from_str::<Value>(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ"
            }"#,
        )
        .unwrap();

        assert_eq!(vm_id, "did:web:mediators-r-us.com#keys-3");
        assert_eq!(
            json_canon::to_string(&jwk).unwrap(),
            json_canon::to_string(&expected_jwk).unwrap()
        );
    }
}

#[cfg(test)]
pub(crate) fn dotenv_flow_read(key: &str) -> Option<String> {
    dotenv_flow::dotenv_iter().unwrap().find_map(|item| {
        let (k, v) = item.unwrap();
        (k == key).then_some(v)
    })
}
