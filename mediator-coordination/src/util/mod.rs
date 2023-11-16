#![allow(unused)]

use did_endpoint::util::{
    filesystem::FileSystem,
    keystore::{KeyStore, KeyStoreError},
};
use did_utils::{
    didcore::{AssertionMethod, Document, KeyAgreement, KeyFormat, VerificationMethod},
    key_jwk::jwk::Jwk,
};
use serde_json::Error as SerdeError;
use std::io;

#[cfg(test)]
use std::io::{Error as IoError, ErrorKind, Result as IoResult};

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
pub fn read_diddoc(
    fs: &mut dyn FileSystem,
    storage_dirpath: &str,
) -> Result<Document, DidDocError> {
    let didpath = format!("{storage_dirpath}/did.json");
    let content = fs.read_to_string(&didpath)?;
    serde_json::from_str(&content).map_err(Into::into)
}

/// Parse key store expected to exist on filesystem.
pub fn read_keystore<'a>(
    fs: &'a mut dyn FileSystem,
    storage_dirpath: &str,
) -> Result<KeyStore<'a>, KeyStoreError> {
    KeyStore::latest(fs, storage_dirpath)
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

    #[test]
    fn can_read_persisted_entities() {
        let mut mock_fs = MockFileSystem;
        assert!(read_diddoc(&mut mock_fs, "").is_ok());
        assert!(read_keystore(&mut mock_fs, "").is_ok());
    }

    #[test]
    fn can_extract_assertion_key() {
        let mut mock_fs = MockFileSystem;
        let diddoc = read_diddoc(&mut mock_fs, "").unwrap();

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
        let mut mock_fs = MockFileSystem;
        let diddoc = read_diddoc(&mut mock_fs, "").unwrap();

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
#[derive(Default)]
pub struct MockFileSystem;

#[cfg(test)]
impl FileSystem for MockFileSystem {
    fn read_to_string(&self, path: &str) -> IoResult<String> {
        match path {
            p if p.ends_with("did.json") => {
                Ok(include_str!("../../test/storage/did.json").to_string())
            }
            p if p.contains("keystore") => {
                Ok(include_str!("../../test/storage/keystore/1697624245.json").to_string())
            }
            _ => Err(IoError::new(ErrorKind::NotFound, "NotFound")),
        }
    }

    fn write(&mut self, path: &str, content: &str) -> IoResult<()> {
        Ok(())
    }

    fn read_dir_files(&self, _path: &str) -> IoResult<Vec<String>> {
        Ok(vec!["/keystore/1697624245.json".to_string()])
    }

    fn create_dir_all(&mut self, _path: &str) -> IoResult<()> {
        Ok(())
    }
}
