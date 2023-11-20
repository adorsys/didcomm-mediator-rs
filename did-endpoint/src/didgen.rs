use crate::util::{didweb, filesystem::StdFileSystem, keystore::KeyStore};
use did_utils::{
    didcore::{
        AssertionMethod, Authentication, Document, KeyAgreement, KeyFormat, Service,
        VerificationMethod,
    },
    key_jwk::jwk::Jwk,
    ldmodel::Context,
};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("KeyGenerationError")]
    KeyGenerationError,
    #[error("MissingServerPublicDomain")]
    MissingServerPublicDomain,
    #[error("DidAddressDerivationError")]
    DidAddressDerivationError,
    #[error("PersistenceError")]
    PersistenceError,
    #[error("Generic: {0}")]
    Generic(String),
}

/// Generates keys and forward them for DID generation
///
/// All persistence is handled at `storage_dirpath`.
pub fn didgen(storage_dirpath: &str, server_public_domain: &str) -> Result<Document, Error> {
    // Create a new store, which is timestamp-aware
    let mut fs = StdFileSystem;
    let mut store = KeyStore::new(&mut fs, storage_dirpath);
    tracing::info!("keystore: {}", store.path());

    // Generate authentication key
    tracing::debug!("generating authentication key");
    let authentication_key = store
        .gen_ed25519_jwk()
        .map_err(|_| Error::KeyGenerationError)?;

    // Generate assertion key
    tracing::debug!("generating assertion key");
    let assertion_key = store
        .gen_ed25519_jwk()
        .map_err(|_| Error::KeyGenerationError)?;

    // Generate agreement key
    tracing::debug!("generating agreement key");
    let agreement_key = store
        .gen_x25519_jwk()
        .map_err(|_| Error::KeyGenerationError)?;

    // Build DID document
    let diddoc = gen_diddoc(
        storage_dirpath,
        server_public_domain,
        authentication_key,
        assertion_key,
        agreement_key,
    )?;

    // Mark successful completion
    tracing::debug!("successful completion");
    Ok(diddoc)
}

/// Builds and persists DID document
fn gen_diddoc(
    storage_dirpath: &str,
    server_public_domain: &str,
    authentication_key: Jwk,
    assertion_key: Jwk,
    agreement_key: Jwk,
) -> Result<Document, Error> {
    tracing::info!("building DID document");

    // Prepare DID address

    let did = didweb::url_to_did_web_id(server_public_domain)
        .map_err(|_| Error::DidAddressDerivationError)?;

    // Prepare authentication verification method

    let authentication_method = VerificationMethod {
        public_key: Some(KeyFormat::Jwk(authentication_key)),
        ..VerificationMethod::new(
            did.clone() + "#keys-1",
            String::from("JsonWebKey2020"),
            did.clone(),
        )
    };

    // Prepare assertion verification method

    let assertion_method = VerificationMethod {
        public_key: Some(KeyFormat::Jwk(assertion_key)),
        ..VerificationMethod::new(
            did.clone() + "#keys-2",
            String::from("JsonWebKey2020"),
            did.clone(),
        )
    };

    // Prepare key agreement verification method

    let agreement_method = VerificationMethod {
        public_key: Some(KeyFormat::Jwk(agreement_key)),
        ..VerificationMethod::new(
            did.clone() + "#keys-3",
            String::from("JsonWebKey2020"),
            did.clone(),
        )
    };

    // Prepare service endpoint

    let service = Service::new(
        did.clone() + "#pop-domain",
        String::from("LinkedDomains"),
        format!("{server_public_domain}/.well-known/did/pop.json"),
    );

    // Build document

    let context = Context::SetOfString(vec![
        String::from("https://www.w3.org/ns/did/v1"),
        String::from("https://w3id.org/security/suites/jws-2020/v1"),
    ]);

    let diddoc = Document {
        authentication: Some(vec![Authentication::Reference(
            authentication_method.id.clone(), //
        )]),
        assertion_method: Some(vec![AssertionMethod::Reference(
            assertion_method.id.clone(), //
        )]),
        key_agreement: Some(vec![KeyAgreement::Reference(
            agreement_method.id.clone(), //
        )]),
        verification_method: Some(vec![
            authentication_method,
            assertion_method,
            agreement_method,
        ]),
        service: Some(vec![service]),
        ..Document::new(context, did)
    };

    // Serialize and persist to file

    let did_json = serde_json::to_string_pretty(&diddoc).unwrap();

    std::fs::create_dir_all(storage_dirpath).map_err(|_| Error::PersistenceError)?;
    std::fs::write(format!("{storage_dirpath}/did.json"), did_json)
        .map_err(|_| Error::PersistenceError)?;

    tracing::info!("persisted DID document to disk");
    Ok(diddoc)
}

/// Validates the integrity of the persisted diddoc
pub fn validate_diddoc(storage_dirpath: &str) -> Result<(), String> {
    // Validate that did.json exists

    let didpath = format!("{storage_dirpath}/did.json");
    if !Path::new(&didpath).exists() {
        return Err(String::from("Missing did.json"));
    };

    // Validate that keystore exists

    let mut fs = StdFileSystem;
    let store = KeyStore::latest(&mut fs, storage_dirpath);
    if store.is_err() {
        return Err(String::from("Missing keystore"));
    }

    // Validate that did.json matches keystore

    let store = store.unwrap();

    let diddoc: Document = match std::fs::read_to_string(didpath) {
        Err(_) => return Err(String::from("Unreadable did.json")),
        Ok(content) => {
            serde_json::from_str(&content).map_err(|_| String::from("Unparseable did.json"))?
        }
    };

    for method in diddoc.verification_method.unwrap_or(vec![]) {
        let pubkey = method.public_key.ok_or(String::from("Missing key"))?;
        let pubkey = match pubkey {
            KeyFormat::Jwk(jwk) => jwk,
            _ => return Err(String::from("Unsupported key format")),
        };

        store
            .find_keypair(&pubkey)
            .ok_or(String::from("Keystore mismatch"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::dotenv_flow_read;

    use did_utils::key_jwk::{
        bytes::Bytes, jwk::Jwk, key::Key, okp::Okp, okp::OkpCurves, prm::Parameters,
    };

    fn setup() -> (String, String) {
        let storage_dirpath = dotenv_flow_read("STORAGE_DIRPATH")
            .map(|p| format!("{}/{}", p, uuid::Uuid::new_v4()))
            .unwrap();

        let server_public_domain = dotenv_flow_read("SERVER_PUBLIC_DOMAIN").unwrap();

        (storage_dirpath, server_public_domain)
    }

    fn cleanup(storage_dirpath: &str) {
        std::fs::remove_dir_all(storage_dirpath).unwrap();
    }

    // Verifies that the didgen function returns a DID document.
    // Does not validate the DID document.
    #[test]
    fn test_didgen() {
        let (storage_dirpath, server_public_domain) = setup();

        let diddoc = didgen(&storage_dirpath, &server_public_domain).unwrap();
        assert_eq!(diddoc.id, "did:web:example.com");

        cleanup(&storage_dirpath);
    }

    // Produces did doc from keys and validate that corresponding verification methods are present.
    #[test]
    fn test_gen_diddoc() {
        let (storage_dirpath, server_public_domain) = setup();

        let authentication_key = Jwk {
            key: Key::Okp(Okp {
                crv: OkpCurves::Ed25519,
                x: Bytes::from(
                    String::from(
                        "d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259",
                    )
                    .into_bytes(),
                ),
                d: None,
            }),
            prm: Parameters::default(),
        };

        let assertion_key = Jwk {
            key: Key::Okp(Okp {
                crv: OkpCurves::Ed25519,
                x: Bytes::from(
                    String::from(
                        "d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259",
                    )
                    .into_bytes(),
                ),
                d: None,
            }),
            prm: Parameters::default(),
        };

        let agreement_key = Jwk {
            key: Key::Okp(Okp {
                crv: OkpCurves::X25519,
                x: Bytes::from(
                    String::from(
                        "d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259",
                    )
                    .into_bytes(),
                ),
                d: None,
            }),
            prm: Parameters::default(),
        };

        let diddoc = gen_diddoc(
            &storage_dirpath,
            &server_public_domain,
            authentication_key.clone(),
            assertion_key.clone(),
            agreement_key.clone(),
        )
        .unwrap();

        // Verify that the DID contains exactly the defined verification methods.
        let expected_verification_methods = vec![
            VerificationMethod {
                id: "did:web:example.com#keys-1".to_string(),
                public_key: Some(KeyFormat::Jwk(authentication_key)),
                ..VerificationMethod::new(
                    "did:web:example.com#keys-1".to_string(),
                    String::from("JsonWebKey2020"),
                    "did:web:example.com".to_string(),
                )
            },
            VerificationMethod {
                id: "did:web:example.com#keys-2".to_string(),
                public_key: Some(KeyFormat::Jwk(assertion_key)),
                ..VerificationMethod::new(
                    "did:web:example.com#keys-2".to_string(),
                    String::from("JsonWebKey2020"),
                    "did:web:example.com".to_string(),
                )
            },
            VerificationMethod {
                id: "did:web:example.com#keys-3".to_string(),
                public_key: Some(KeyFormat::Jwk(agreement_key)),
                ..VerificationMethod::new(
                    "did:web:example.com#keys-3".to_string(),
                    String::from("JsonWebKey2020"),
                    "did:web:example.com".to_string(),
                )
            },
        ];

        let actual_verification_methods = diddoc.verification_method.unwrap();

        let actual = json_canon::to_string(&actual_verification_methods).unwrap();
        let expected = json_canon::to_string(&expected_verification_methods).unwrap();
        assert_eq!(expected, actual);

        cleanup(&storage_dirpath);
    }

    #[test]
    fn test_validate_diddoc() {
        let (storage_dirpath, server_public_domain) = setup();

        didgen(&storage_dirpath, &server_public_domain).unwrap();
        assert!(validate_diddoc(&storage_dirpath).is_ok());

        cleanup(&storage_dirpath);
    }
}
