#![allow(unused)]

use did_endpoint::util::KeyStore;
use did_utils::didcore::{
    AssertionMethod, Document, Jwk, KeyAgreement, KeyFormat, VerificationMethod,
};

/// Parse DID document expected to exist on filesystem.
pub fn read_diddoc(storage_dirpath: &str) -> Option<Document> {
    let didpath = format!("{storage_dirpath}/did.json");
    std::fs::read_to_string(didpath)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

/// Parse key store expected to exist on filesystem.
pub fn read_keystore(storage_dirpath: &str) -> Option<KeyStore> {
    KeyStore::latest(storage_dirpath)
}

/// Search an assertion key in a DID document.
///
/// Return its verification method ID and its Jwk
/// representation if present, else Option::None.
pub fn extract_assertion_key(diddoc: &Document) -> Option<(String, Jwk)> {
    let id = match &diddoc.assertion_method {
        None => return None,
        Some(methods) => match methods.get(0) {
            None => return None,
            Some(method) => match method {
                AssertionMethod::Reference(reference) => reference,
                AssertionMethod::Embedded(vm) => return extract_public_jwk_from_vm(vm),
            },
        },
    };

    diddoc.verification_method.as_ref().and_then(|arr| {
        arr.iter()
            .find(|vm| &vm.id == id)
            .and_then(extract_public_jwk_from_vm)
    })
}

/// Search a agreement key in a DID document.
///
/// Return its verification method ID and its Jwk
/// representation if present, else Option::None.
pub fn extract_agreement_key(diddoc: &Document) -> Option<(String, Jwk)> {
    let id = match &diddoc.key_agreement {
        None => return None,
        Some(methods) => match methods.get(0) {
            None => return None,
            Some(method) => match method {
                KeyAgreement::Reference(reference) => reference,
                KeyAgreement::Embedded(vm) => return extract_public_jwk_from_vm(vm),
            },
        },
    };

    diddoc.verification_method.as_ref().and_then(|arr| {
        arr.iter()
            .find(|vm| &vm.id == id)
            .and_then(extract_public_jwk_from_vm)
    })
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
        assert!(read_diddoc(&storage_dirpath).is_some());
        assert!(read_keystore(&storage_dirpath).is_some());
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
