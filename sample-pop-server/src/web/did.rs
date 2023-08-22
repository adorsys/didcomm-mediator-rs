use std::collections::HashMap;

use axum::{extract::Query, response::Json, routing::get, Router};
use ed25519_dalek::Signer;
use hyper::StatusCode;
use multibase::Base::Base58Btc;
use serde_json::{json, Value};
use ssi::{
    did::VerificationRelationship,
    ldp::{dataintegrity::DataIntegrityCryptoSuite, Proof, ProofSuiteType},
    vc::{
        Context, Contexts, Credential, CredentialSubject, Issuer, OneOrMany, StringOrURI,
        DEFAULT_CONTEXT_V2, URI,
    },
};

use crate::util::keystore::KeyStore;
use crate::DIDDOC_DIR;

pub fn routes() -> Router {
    Router::new() //
        .route("/.well-known/did.json", get(diddoc))
        .route("/.well-known/did/pop.json", get(didpop))
}

pub async fn diddoc() -> Result<Json<Value>, StatusCode> {
    match tokio::fs::read_to_string(DIDDOC_DIR.to_owned() + "/did.json").await {
        Ok(content) => Ok(Json(serde_json::from_str(&content).unwrap())),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn didpop(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let challenge = params.get("challenge").ok_or(StatusCode::BAD_REQUEST)?;
    let keystore = KeyStore::latest().expect("Missing keystore file");

    // Read secret used for key encryption

    let secret = std::env::var("DIDGEN_SECRET").expect("Could not find secret key.");

    // Load DID document and its verification methods

    let diddoc = &diddoc().await?.0;
    let did_address = diddoc.get("id").unwrap().as_str().unwrap();
    let methods = diddoc
        .get("verificationMethod")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| {
            if let Some(type_) = x.get("type") {
                type_.as_str().unwrap() == "Ed25519VerificationKey2020"
            } else {
                false
            }
        });

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
        property_set: serde_json::from_str(&diddoc.to_string()).unwrap(),
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

    for method in methods {
        // Lookup signing key from keystore
        let pubkey = method.get("publicKeyMultibase").unwrap().as_str().unwrap();
        let signing_key = keystore
            .lookup_signing_key(pubkey, &secret)
            .expect("Missing key");

        // Charter proof without adding signature
        let mut proof = Proof {
            cryptosuite: Some(DataIntegrityCryptoSuite::JcsEddsa2022),
            created: Some(ssi::ldp::now_ms()),
            challenge: Some(challenge.clone()),
            creator: Some(did_address.to_owned()),
            proof_purpose: Some(VerificationRelationship::default()),
            verification_method: Some(method.get("id").unwrap().as_str().unwrap().to_owned()),
            ..Proof::new(ProofSuiteType::DataIntegrityProof)
        };

        // Add the incomplete proof block to the credential and produce a canonical form
        let mut tmp_vc = vc.clone();
        tmp_vc.add_proof(proof.clone());
        let message = json_canon::to_string(&tmp_vc).unwrap();

        // Compute digital signature
        let signature = signing_key.sign(message.as_bytes()).to_string();
        let signature = multibase::encode(Base58Btc, signature.as_bytes());

        // Add digital signature to proof
        proof.proof_value = Some(signature);
        vec_proof.push(proof);
    }

    // Insert all proofs
    vc.proof = Some(OneOrMany::Many(vec_proof));

    // Output final verifiable credential
    Ok(Json(json!(vc)))
}
