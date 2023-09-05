use std::path::Path;

use crate::{
    util::{didweb, KeyStore},
    DIDDOC_DIR,
};
use ssi::{
    did::{
        Context, Contexts, Document, DocumentBuilder, Service, ServiceEndpoint, VerificationMethod,
        VerificationMethodMap, DEFAULT_CONTEXT, DIDURL,
    },
    jwk::JWK,
    one_or_many::OneOrMany,
};

#[derive(Debug)]
pub enum Error {
    KeyGenerationError,
    MissingServerPublicDomain,
    DidAddressDerivationError,
    PersistenceError,
    Unknown(String),
}

/// Generates keys and forward them for DID generation
pub fn didgen() -> Result<String, Error> {
    // Create a new store, which is timestamp-aware
    let mut store = KeyStore::new();
    tracing::info!("Keystore: {}", store.path());

    // Generate authentication key
    tracing::info!("Generating authentication key...");
    let authentication_key = store
        .gen_ed25519_jwk()
        .map_err(|_| Error::KeyGenerationError)?;

    // Generate assertion key
    tracing::info!("Generating assertion key...");
    let assertion_key = store
        .gen_ed25519_jwk()
        .map_err(|_| Error::KeyGenerationError)?;

    // Generate agreement key
    tracing::info!("Generating agreement key...");
    let agreement_key = store
        .gen_x25519_jwk()
        .map_err(|_| Error::KeyGenerationError)?;

    // Build DID document
    let diddoc = gen_diddoc(authentication_key, assertion_key, agreement_key)?;

    // Mark successful completion
    tracing::info!("Successful completion.");
    Ok(diddoc)
}

/// Builds and persists DID document
fn gen_diddoc(
    authentication_key: JWK,
    assertion_key: JWK,
    agreement_key: JWK,
) -> Result<String, Error> {
    tracing::info!("Building DID document...");

    // Prepare DID address

    let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN") //
        .map_err(|_| Error::MissingServerPublicDomain)?;
    let did = didweb::url_to_did_web_id(&public_domain) //
        .map_err(|_| Error::DidAddressDerivationError)?;

    // Prepare authentication verification method

    let authentication_method_id = DIDURL::try_from(did.clone() + "#keys-1").unwrap();

    let authentication_method = VerificationMethodMap {
        id: authentication_method_id.to_string(),
        controller: did.clone(),
        type_: String::from("JsonWebKey2020"),
        public_key_jwk: Some(authentication_key),
        ..Default::default()
    };

    // Prepare assertion verification method

    let assertion_method_id = DIDURL::try_from(did.clone() + "#keys-2").unwrap();

    let assertion_method = VerificationMethodMap {
        id: assertion_method_id.to_string(),
        controller: did.clone(),
        type_: String::from("JsonWebKey2020"),
        public_key_jwk: Some(assertion_key),
        ..Default::default()
    };

    // Prepare key agreement verification method

    let agreement_method_id = DIDURL::try_from(did.clone() + "#keys-3").unwrap();

    let agreement_method = VerificationMethodMap {
        id: agreement_method_id.to_string(),
        controller: did.clone(),
        type_: String::from("JsonWebKey2020"),
        public_key_jwk: Some(agreement_key),
        ..Default::default()
    };

    // Prepare service endpoint

    let service_id = DIDURL::try_from(did.clone() + "#pop-domain").unwrap();

    let service = Service {
        id: service_id.to_string(),
        type_: OneOrMany::One(String::from("LinkedDomains")),
        service_endpoint: Some(OneOrMany::One(ServiceEndpoint::URI(format!(
            "{public_domain}/.well-known/did/pop.json"
        )))),
        property_set: None,
    };

    // Build document

    let doc = DocumentBuilder::default()
        .context(Contexts::Many(vec![
            Context::URI(DEFAULT_CONTEXT.to_owned().into()),
            Context::URI(
                "https://w3id.org/security/suites/jws-2020/v1"
                    .parse()
                    .unwrap(),
            ),
        ]))
        .id(did)
        .authentication(vec![VerificationMethod::DIDURL(authentication_method_id)])
        .assertion_method(vec![VerificationMethod::DIDURL(assertion_method_id)])
        .key_agreement(vec![VerificationMethod::DIDURL(agreement_method_id)])
        .verification_method(vec![
            VerificationMethod::Map(authentication_method),
            VerificationMethod::Map(assertion_method),
            VerificationMethod::Map(agreement_method),
        ])
        .service(vec![service])
        .build()
        .unwrap();

    // Serialize and persist to file

    let did_json = serde_json::to_string_pretty(&doc).unwrap();

    std::fs::write(format!("{DIDDOC_DIR}/did.json"), &did_json)
        .map_err(|_| Error::PersistenceError)?;

    tracing::info!("Persisted DID document to file.");
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

    let methods = match &diddoc.verification_method {
        None => vec![],
        Some(data) => data
            .iter()
            .filter_map(|x| match x {
                VerificationMethod::Map(map) => Some(map),
                _ => unreachable!(),
            })
            .collect(),
    };

    for method in methods {
        let pubkey = method.public_key_jwk.as_ref().unwrap();
        store
            .find_keypair(pubkey)
            .ok_or(String::from("Keystore mismatch"))?;
    }

    Ok(())
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
