use crate::plugin::didpop::{
    util::{didweb, KeyStore},
    DIDDOC_DIR,
};
use did_utils::{
    didcore::{
        AssertionMethod, Authentication, Document, Jwk, KeyAgreement, KeyFormat, Service,
        VerificationMethod,
    },
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
pub fn didgen() -> Result<String, Error> {
    // Create a new store, which is timestamp-aware
    let mut store = KeyStore::new();
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
    let diddoc = gen_diddoc(authentication_key, assertion_key, agreement_key)?;

    // Mark successful completion
    tracing::debug!("successful completion");
    Ok(diddoc)
}

/// Builds and persists DID document
fn gen_diddoc(
    authentication_key: Jwk,
    assertion_key: Jwk,
    agreement_key: Jwk,
) -> Result<String, Error> {
    tracing::info!("building DID document");

    // Prepare DID address

    let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN") //
        .map_err(|_| Error::MissingServerPublicDomain)?;
    let did = didweb::url_to_did_web_id(&public_domain) //
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
        format!("{public_domain}/.well-known/did/pop.json"),
    );

    // Build document

    let context = Context::SetOfString(vec![
        String::from("https://www.w3.org/ns/did/v1"),
        String::from("https://w3id.org/security/suites/jws-2020/v1"),
    ]);

    let doc = Document {
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

    let did_json = serde_json::to_string_pretty(&doc).unwrap();

    std::fs::write(format!("{DIDDOC_DIR}/did.json"), &did_json)
        .map_err(|_| Error::PersistenceError)?;

    tracing::info!("persisted DID document to disk");
    Ok(did_json)
}

/// Validates the integrity of the persisted diddoc
pub fn validate_diddoc() -> Result<(), String> {
    // Validate that did.json exists

    let didpath = format!("{DIDDOC_DIR}/did.json");
    if !Path::new(&didpath).exists() {
        return Err(String::from("Missing did.json"));
    };

    // Validate that keystore exists

    let store = KeyStore::latest();
    if store.is_none() {
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
    fn setup() {
        std::env::set_var("SERVER_PUBLIC_DOMAIN", "example.com");
    }
    
    // Verifies that the didgen function returns a DID document.
    // Does not validate the DID document.
    #[test]
    fn test_didgen() {
        setup();
        let diddoc = didgen().unwrap();
        assert_ne!(diddoc, "");
    }

    // Produces did doc from keys and validate that corresponding verification methods are present.
    #[test]
    fn test_gen_diddoc() {
        setup();
        let authentication_key = Jwk {
            key_id: None,
            key_type: String::from("OKP"),
            curve: String::from("Ed25519"),
            x: Some(String::from("d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259")),
            y: None,
            d: None,
        };

        let assertion_key = Jwk {
            key_id: None,
            key_type: String::from("OKP"),
            curve: String::from("Ed25519"),
            x: Some(String::from("d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259")),
            y: None,
            d: None,
        };

        let agreement_key = Jwk {
            key_id: None,
            key_type: String::from("OKP"),
            curve: String::from("X25519"),
            x: Some(String::from("d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259")),
            y: None,
            d: None,
        };

        let diddoc = gen_diddoc(authentication_key.clone(), assertion_key.clone(), agreement_key.clone()).unwrap();
        assert_ne!(diddoc, "");

        let diddoc_value: Document = serde_json::from_str(&diddoc).unwrap();

        // Verify that the DID contains exactly the defined verification methods.
        let expected_verification_methods = vec![
            VerificationMethod {
                id: "did:web:example.com#keys-1".to_string(),
                public_key: Some(KeyFormat::Jwk(authentication_key)),
                ..VerificationMethod::new("did:web:example.com#keys-1".to_string(), String::from("JsonWebKey2020"), "did:web:example.com".to_string())
            },
            VerificationMethod {
                id: "did:web:example.com#keys-2".to_string(),
                public_key: Some(KeyFormat::Jwk(assertion_key)),
                ..VerificationMethod::new("did:web:example.com#keys-2".to_string(), String::from("JsonWebKey2020"), "did:web:example.com".to_string())
            },
            VerificationMethod {
                id: "did:web:example.com#keys-3".to_string(),
                public_key: Some(KeyFormat::Jwk(agreement_key)),
                ..VerificationMethod::new("did:web:example.com#keys-3".to_string(), String::from("JsonWebKey2020"), "did:web:example.com".to_string())
            },
        ];

        let actual_verification_methods = diddoc_value.verification_method.unwrap();

        let actual = json_canon::to_string(&actual_verification_methods).unwrap();
        let expected = json_canon::to_string(&expected_verification_methods).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_validate_diddoc() {
        setup();
        // Create a new store, which is timestamp-aware
        let mut store = KeyStore::new();
        tracing::info!("keystore: {}", store.path());

        // Generate authentication key
        tracing::debug!("generating authentication key");
        let authentication_key = store.gen_ed25519_jwk().map_err(|_| Error::KeyGenerationError).unwrap();

        // Generate assertion key
        tracing::debug!("generating assertion key");
        let assertion_key = store.gen_ed25519_jwk().map_err(|_| Error::KeyGenerationError).unwrap();

        // Generate agreement key
        tracing::debug!("generating agreement key");
        let agreement_key = store.gen_x25519_jwk().map_err(|_| Error::KeyGenerationError).unwrap();

        // Build DID document
        let did_json = gen_diddoc(authentication_key, assertion_key, agreement_key).unwrap();

        std::fs::write(format!("{DIDDOC_DIR}/did.json"), &did_json).unwrap();

        // Validate DID document
        assert!(validate_diddoc().is_ok());
    }
}
