use std::collections::HashMap;

use axum::extract::Query;
use axum::routing::get;
use axum::{response::Json, Router};
use ed25519_dalek::Signer;
use hyper::StatusCode;
use serde_json::{json, Value};

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
    let mut proof = vec![];

    // Read secret used for key encryption
    let secret = std::env::var("DIDGEN_SECRET").expect("Could not find secret key.");

    let diddoc = &diddoc().await?.0;
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

    for method in methods {
        let pubkey = method.get("publicKeyMultibase").unwrap().as_str().unwrap();
        let signing_key = keystore
            .lookup_signing_key(pubkey, &secret)
            .expect("Missing key");
        let signature = signing_key.sign(challenge.as_bytes()).to_string();

        proof.push(json!({
            "verifyingKey": pubkey,
            "signature": signature,
        }));
    }

    Ok(Json(json!({
        "challenge": challenge,
        "proof": proof,
    })))
}
