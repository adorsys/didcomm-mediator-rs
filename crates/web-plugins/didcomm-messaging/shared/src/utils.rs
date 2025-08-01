pub mod resolvers;
pub mod tests_utils;

use did_utils::{
    didcore::{Document, KeyFormat, VerificationMethod, VerificationMethodType},
    jwk::Jwk,
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
pub fn extract_authentication_key(diddoc: &Document) -> Option<(String, Jwk)> {
    let method = diddoc.authentication.as_ref()?.first()?;
    extract_key_from_diddoc!(VerificationMethodType)(diddoc, method)
}

/// Search an agreement key in a DID document.
///
/// if present, return its verification method ID and JWK representation.
pub fn extract_agreement_key(diddoc: &Document) -> Option<(String, Jwk)> {
    let method = diddoc.key_agreement.as_ref()?.first()?;
    extract_key_from_diddoc!(VerificationMethodType)(diddoc, method)
}

/// Reads public JWK from verification method.
///
/// Return verification method ID and JWK if present, else Option::None.
fn extract_public_jwk_from_vm(vm: &VerificationMethod) -> Option<(String, Jwk)> {
    vm.public_key.as_ref().and_then(|key| match key {
        KeyFormat::Jwk(jwk) => Some((vm.id.clone(), *jwk.clone())),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn can_extract_authentication_key() {
        let diddoc = tests_utils::tests::setup().diddoc.clone();

        let (vm_id, jwk) = extract_authentication_key(&diddoc).unwrap();
        let expected_jwk = serde_json::from_str::<Value>(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "PuG2L5um-tAnHlvT29gTm9Wj9fZca16vfBCPKsHB5cA"
            }"#,
        )
        .unwrap();

        assert_eq!(vm_id, "#key-2");
        assert_eq!(
            json_canon::to_string(&jwk).unwrap(),
            json_canon::to_string(&expected_jwk).unwrap()
        );
    }

    #[test]
    fn can_extract_agreement_key() {
        let diddoc = tests_utils::tests::setup().diddoc.clone();

        let (vm_id, jwk) = extract_agreement_key(&diddoc).unwrap();
        let expected_jwk = serde_json::from_str::<Value>(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "_EgIPSRgbPPw5-nUsJ6xqMvw5rXn3BViGADeUrjAMzA"
            }"#,
        )
        .unwrap();

        assert_eq!(vm_id, "#key-1");
        assert_eq!(
            json_canon::to_string(&jwk).unwrap(),
            json_canon::to_string(&expected_jwk).unwrap()
        );
    }
}
