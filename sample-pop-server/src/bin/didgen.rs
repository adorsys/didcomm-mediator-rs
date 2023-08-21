use std::collections::BTreeMap;

use sample_pop_server::util::KeyStore;
use sample_pop_server::{util, DIDDOC_DIR};
use serde_json::{json, Value};
use ssi::did::{
    Context, Contexts, DocumentBuilder, Service, ServiceEndpoint, VerificationMethod,
    VerificationMethodMap, DEFAULT_CONTEXT, DIDURL,
};
use ssi::one_or_many::OneOrMany;

/// Program entry
fn main() -> std::io::Result<()> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable tracing
    tracing_subscriber::fmt::init();

    // Read secret for key encryption
    let secret = std::env::var("DIDGEN_SECRET").expect("Please provide a secret key.");

    // Init store with timestamp-aware path
    let store = KeyStore::new();
    tracing::info!("keystore: {}", store.path());

    // Generate authentication key
    tracing::info!("Generating authentication key...");
    let authentication_key = store.gen_signing_keys(&secret);

    // Generate assertion key
    tracing::info!("Generating assertion key...");
    let assertion_key = store.gen_signing_keys(&secret);

    // Build DID document
    gen_diddoc(&authentication_key, &assertion_key);

    tracing::info!("Successful completion.");
    Ok(())
}

/// Builds and persists DID document
fn gen_diddoc(authentication_key: &String, assertion_key: &String) {
    tracing::info!("Building DID document...");

    // Prepare DID address

    let public_domain = std::env::var("SERVER_PUBLIC_DOMAIN") //
        .expect("Missing SERVER_PUBLIC_DOMAIN");
    let did = util::url_to_did_web_id(&public_domain) //
        .expect("Error deriving did:web address");

    // Prepare authentication verification method

    let authentication_method_id = DIDURL::try_from(did.clone() + "#keys-1").unwrap();

    let mut authentication_property_set = BTreeMap::<String, Value>::new();
    authentication_property_set.insert(
        String::from("publicKeyMultibase"),
        json!(authentication_key),
    );

    let authentication_method = VerificationMethodMap {
        id: authentication_method_id.to_string(),
        controller: did.clone(),
        type_: String::from("Ed25519VerificationKey2020"),
        property_set: Some(authentication_property_set),
        ..Default::default()
    };

    // Prepare assertion verification method

    let assertion_method_id = DIDURL::try_from(did.clone() + "#keys-2").unwrap();

    let mut assertion_property_set = BTreeMap::<String, Value>::new();
    assertion_property_set.insert(String::from("publicKeyMultibase"), json!(assertion_key));

    let assertion_method = VerificationMethodMap {
        id: assertion_method_id.to_string(),
        controller: did.clone(),
        type_: String::from("Ed25519VerificationKey2020"),
        property_set: Some(assertion_property_set),
        ..Default::default()
    };

    // Prepare service endpoint

    let service_id = DIDURL::try_from(did.clone() + "#pop-domain").unwrap();

    let service = Service {
        id: service_id.to_string(),
        type_: OneOrMany::One(String::from("LinkedDomains")),
        service_endpoint: Some(OneOrMany::One(ServiceEndpoint::URI(format!(
            "{public_domain}/did/pop"
        )))),
        property_set: None,
    };

    // Build document

    let doc = DocumentBuilder::default()
        .context(Contexts::Many(vec![
            Context::URI(DEFAULT_CONTEXT.to_owned().into()),
            Context::URI(
                "https://w3id.org/security/suites/ed25519-2020/v1"
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

#[cfg(test)]
mod tests {
    #[test]
    fn can_jcs_serialize() {
        let data = serde_json::json!({
            "from_account": "543 232 625-3",
            "to_account": "321 567 636-4",
            "amount": 500.50,
            "currency": "USD"
        });

        let jcs = r#"{"amount":500.5,"currency":"USD","from_account":"543 232 625-3","to_account":"321 567 636-4"}"#;

        assert_eq!(jcs, json_canon::to_string(&data).unwrap());
    }
}
