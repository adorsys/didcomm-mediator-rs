use axum::{extract::Query, response::Json, routing::get, Router};
use hyper::StatusCode;
use serde_json::{json, Value};
use ssi::{
    did::{Document, VerificationMethod, VerificationRelationship, DIDURL},
    jsonld::ContextLoader,
    ldp::{dataintegrity::DataIntegrityCryptoSuite, Proof, ProofSuiteType},
    vc::{
        Context, Contexts, Credential, CredentialSubject, Issuer, LinkedDataProofOptions,
        OneOrMany, StringOrURI, DEFAULT_CONTEXT_V2, URI,
    },
};
use std::collections::HashMap;

use crate::{
    util::{resolver::StaticResolver, KeyStore},
    DIDDOC_DIR,
};

pub fn routes() -> Router {
    Router::new() //
        .route("/.well-known/did.json", get(diddoc))
        .route("/.well-known/did/pop.json", get(didpop))
}

pub async fn diddoc() -> Result<Json<Value>, StatusCode> {
    match tokio::fs::read_to_string(&format!("{DIDDOC_DIR}/did.json")).await {
        Ok(content) => Ok(Json(serde_json::from_str(&content).unwrap())),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn didpop(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let challenge = params.get("challenge").ok_or(StatusCode::BAD_REQUEST)?;
    let keystore = KeyStore::latest().expect("Keystore file probably missing");

    // Load DID document and its verification methods

    let diddoc_value = diddoc().await?.0;
    let diddoc: Document = serde_json::from_value(diddoc_value.clone()).unwrap();

    let did_address = diddoc.id.clone();
    let methods = match &diddoc.verification_method {
        None => vec![],
        Some(data) => data
            .iter()
            .filter_map(|x| match x {
                VerificationMethod::Map(map) => Some(map),
                _ => None,
            })
            .collect(),
    };

    // Prepare fields for verifiable credential

    let context = Contexts::Many(vec![Context::URI(URI::String(
        DEFAULT_CONTEXT_V2.to_owned(),
    ))]);

    let type_ = OneOrMany::Many(vec![
        String::from("VerifiableCredential"),
        String::from("DIDDocument"),
    ]);

    let issuer = Some(Issuer::URI(did_address.parse().unwrap()));

    let credential_subject = OneOrMany::One(CredentialSubject {
        id: None,
        property_set: serde_json::from_value(diddoc_value).unwrap(),
    });

    // Build verifiable credential

    let mut vc = Credential {
        id: Some(StringOrURI::URI(URI::String(
            "urn:uuid:".to_string() + &uuid::Uuid::new_v4().to_string(),
        ))),
        property_set: {
            let mut map = HashMap::new();
            map.insert(String::from("validFrom"), json!(ssi::ldp::now_ms()));
            Some(map)
        },
        //
        context,
        type_,
        issuer,
        credential_subject,
        //
        proof: None,
        issuance_date: None,
        expiration_date: None,
        credential_status: None,
        terms_of_use: None,
        evidence: None,
        credential_schema: None,
        refresh_service: None,
    };

    // Generate proofs of possession

    let mut vec_proof: Vec<Proof> = vec![];

    let resolver = StaticResolver::new(&diddoc);
    let mut context_loader = ContextLoader::default();
    let mut options = LinkedDataProofOptions {
        type_: Some(ProofSuiteType::DataIntegrityProof),
        cryptosuite: Some(DataIntegrityCryptoSuite::JcsEddsa2022),
        challenge: Some(challenge.to_string()),
        ..Default::default()
    };

    for method in methods {
        // Lookup keypair from keystore
        let pubkey = method
            .public_key_jwk
            .as_ref()
            .expect("Verification methods should embed JWK public keys.");
        let jwk = keystore.find_keypair(pubkey).expect("Missing key");

        options.verification_method = Some(URI::String(method.id.clone()));
        options.proof_purpose = inspect_vm_relationship(&diddoc, &method.id);

        // Generate proof
        let proof = vc
            .generate_proof(&jwk, &options, &resolver, &mut context_loader)
            .await
            .expect("Error generating proof");

        // Carry proof
        vec_proof.push(proof);
    }

    // Insert all proofs
    vc.proof = Some(OneOrMany::Many(vec_proof));

    // Output final verifiable credential
    Ok(Json(json!(vc)))
}

/// Inspects in a DID document the relationship of
/// a verification method based on its identifier
fn inspect_vm_relationship(
    diddoc: &Document,
    verification_method_id: &str,
) -> Option<VerificationRelationship> {
    let vm_url = &DIDURL::try_from(verification_method_id.to_string()).unwrap();

    if let Some(data) = &diddoc.authentication {
        if data.iter().any(|x| match x {
            VerificationMethod::DIDURL(url) => url == vm_url,
            _ => false,
        }) {
            return Some(VerificationRelationship::Authentication);
        }
    }

    if let Some(data) = &diddoc.assertion_method {
        if data.iter().any(|x| match x {
            VerificationMethod::DIDURL(url) => url == vm_url,
            _ => false,
        }) {
            return Some(VerificationRelationship::AssertionMethod);
        }
    }

    None
}
