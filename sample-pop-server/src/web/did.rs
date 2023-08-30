use axum::{extract::Query, response::Json, routing::get, Router};
use ed25519_dalek::Signer;
use hyper::StatusCode;
use multibase::Base::Base58Btc;
use serde_json::{json, Value};
use ssi::{
    did::VerificationRelationship,
    hash::sha256::sha256,
    ldp::{dataintegrity::DataIntegrityCryptoSuite, Proof, ProofSuiteType},
    vc::{
        Context, Contexts, Credential, CredentialSubject, Issuer, OneOrMany, StringOrURI,
        DEFAULT_CONTEXT_V2, URI,
    },
};
use std::collections::HashMap;

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

        // Build hash message to sign
        let message = [
            sha256(json_canon::to_string(&vc).unwrap().as_bytes()),
            sha256(json_canon::to_string(&proof).unwrap().as_bytes()),
        ]
        .concat();

        // Compute digital signature
        let signature = signing_key.sign(&message).to_bytes();
        let signature = multibase::encode(Base58Btc, signature);

        // Add digital signature to proof
        proof.proof_value = Some(signature);
        vec_proof.push(proof);
    }

    // Insert all proofs
    vc.proof = Some(OneOrMany::Many(vec_proof));

    // Output final verifiable credential
    Ok(Json(json!(vc)))
}

#[cfg(test)]
mod tests {
    use crate::app;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use ssi::{
        hash::sha256::sha256,
        ldp::Proof,
        vc::{Credential, OneOrMany},
    };
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn verify_didpop() {
        let app = app();
        dotenv_flow::dotenv_flow().ok();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/.well-known/did/pop.json?challenge={}",
                        uuid::Uuid::new_v4().to_string()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let mut vc: Credential = serde_json::from_slice(&body).unwrap();

        // Extract proofs
        let proofs = match vc.proof.unwrap() {
            OneOrMany::Many(proof) => proof,
            OneOrMany::One(proof) => vec![proof],
        };

        // Remove all proofs from VC
        vc.proof = None;

        // Verify all proofs
        for proof in proofs {
            let verification_method = proof.verification_method.as_ref().unwrap();
            if let OneOrMany::One(credential_subject) = &vc.credential_subject {
                let pubkey = credential_subject
                    .property_set
                    .as_ref()
                    .unwrap()
                    .get("verificationMethod")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x.get("id").unwrap().as_str().unwrap() == verification_method)
                    .map(|x| x.get("publicKeyMultibase").unwrap())
                    .unwrap()
                    .as_str()
                    .unwrap();

                let pubkey = multibase::decode(pubkey).unwrap().1;
                let pubkey: [u8; 32] = pubkey[..32].try_into().unwrap();
                let verifying_key = VerifyingKey::from_bytes(&pubkey).unwrap();

                let message = [
                    sha256(json_canon::to_string(&vc).unwrap().as_bytes()),
                    sha256(
                        json_canon::to_string(&Proof {
                            proof_value: None,
                            ..proof
                        })
                        .unwrap()
                        .as_bytes(),
                    ),
                ]
                .concat();

                let proof_value = proof.proof_value.as_ref().unwrap();
                let signature = multibase::decode(proof_value).unwrap().1;
                let signature: [u8; 64] = signature[..64].try_into().unwrap();

                assert!(verifying_key
                    .verify(&message, &Signature::from_bytes(&signature))
                    .is_ok());
            } else {
                panic!();
            }
        }
    }
}
