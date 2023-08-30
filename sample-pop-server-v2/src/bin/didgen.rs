use sample_pop_server::{
    util::{convert, KeyStore},
    DIDDOC_DIR,
};
use ssi::{
    did::{
        Context, Contexts, DocumentBuilder, Service, ServiceEndpoint, VerificationMethod,
        VerificationMethodMap, DEFAULT_CONTEXT, DIDURL,
    },
    jwk::JWK,
    one_or_many::OneOrMany,
};
use std::error::Error;

/// Program entry
fn main() -> Result<(), Box<dyn Error>> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable tracing
    tracing_subscriber::fmt::init();

    // Create a new store, which is timestamp-aware
    let mut store = KeyStore::new();
    tracing::info!("keystore: {}", store.path());

    // Generate authentication key
    tracing::info!("Generating authentication key...");
    let authentication_key = store.gen_ed25519_jwk()?;

    // Generate assertion key
    tracing::info!("Generating assertion key...");
    let assertion_key = store.gen_ed25519_jwk()?;

    // Build DID document
    gen_diddoc(authentication_key, assertion_key);

    tracing::info!("Successful completion.");
    Ok(())
}

/// Builds and persists DID document
fn gen_diddoc(authentication_key: JWK, assertion_key: JWK) {
    tracing::info!("Building DID document...");

    // Prepare DID address

    let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN") //
        .expect("Missing SERVER_PUBLIC_DOMAIN");
    let did = convert::url_to_did_web_id(&public_domain) //
        .expect("Error deriving did:web address");

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
        .verification_method(vec![
            VerificationMethod::Map(authentication_method),
            VerificationMethod::Map(assertion_method),
        ])
        .service(vec![service])
        .build()
        .unwrap();

    // Serialize and persist to file

    let did_json = serde_json::to_string_pretty(&doc).unwrap();

    std::fs::write(DIDDOC_DIR.to_owned() + "/did.json", &did_json)
        .expect("Error persisting JSON document");

    println!("{}", &did_json);
    tracing::info!("Persisted DID document to file.");
}
