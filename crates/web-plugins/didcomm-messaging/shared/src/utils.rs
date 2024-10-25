pub mod resolvers;
pub mod tests_utils;

use did_utils::{
    didcore::{VerificationMethodType, Document, KeyFormat, VerificationMethod},
    jwk::Jwk,
};
use filesystem::FileSystem;
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
pub fn read_diddoc(fs: &dyn FileSystem, storage_dirpath: &str) -> Result<Document, DidDocError> {
    let didpath = format!("{storage_dirpath}/did.json");
    let content = fs.read_to_string(didpath.as_ref())?;
    serde_json::from_str(&content).map_err(Into::into)
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
    extract_key_from_diddoc!(VerificationMethodType)(diddoc, method)
}

/// Search an agreement key in a DID document.
///
/// if present, return its verification method ID and JWK representation.
pub fn extract_agreement_key(diddoc: &Document) -> Option<(String, Jwk)> {
    let method = diddoc.key_agreement.as_ref()?.get(0)?;
    extract_key_from_diddoc!(VerificationMethodType)(diddoc, method)
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
    use filesystem::MockFileSystem;
    use serde_json::Value;

    #[test]
    fn can_extract_assertion_key() {
        let mock_fs = MockFileSystem;
        let diddoc = read_diddoc(&mock_fs, "").unwrap();

        let (vm_id, jwk) = extract_assertion_key(&diddoc).unwrap();
        let expected_jwk = serde_json::from_str::<Value>(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4"
            }"#,
        )
        .unwrap();

        assert_eq!(
            vm_id,
            "did:web:alice-mediator.com:alice_mediator_pub#keys-2"
        );
        assert_eq!(
            json_canon::to_string(&jwk).unwrap(),
            json_canon::to_string(&expected_jwk).unwrap()
        );
    }

    #[test]
    fn can_extract_agreement_key() {
        let mock_fs = MockFileSystem;
        let diddoc = read_diddoc(&mock_fs, "").unwrap();

        let (vm_id, jwk) = extract_agreement_key(&diddoc).unwrap();
        let expected_jwk = serde_json::from_str::<Value>(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ"
            }"#,
        )
        .unwrap();

        assert_eq!(
            vm_id,
            "did:web:alice-mediator.com:alice_mediator_pub#keys-3"
        );
        assert_eq!(
            json_canon::to_string(&jwk).unwrap(),
            json_canon::to_string(&expected_jwk).unwrap()
        );
    }
}
