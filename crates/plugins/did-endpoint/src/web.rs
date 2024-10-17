use axum::{
    extract::{Query, State},
    response::Json,
    routing::get,
    Router,
};
use chrono::Utc;
use did_utils::{
    didcore::{Document, KeyFormat, Proofs}, jwk::Jwk, proof::{CryptoProof, EdDsaJcs2022, Proof, PROOF_TYPE_DATA_INTEGRITY_PROOF}, vc::{VerifiableCredential, VerifiablePresentation}
};
use didcomm::secrets::SecretMaterial;
use hyper::StatusCode;
use mongodb::bson::doc;
use multibase::Base;
use serde_json::{json, Value};
use shared::state::{AppState, AppStateRepository};
use std::{collections::HashMap, sync::Arc};

const DEFAULT_CONTEXT_V2: &str = "https://www.w3.org/ns/credentials/v2";

pub(crate) fn routes(state: Arc<AppState>) -> Router {
    Router::new() //
        .route("/.well-known/did.json", get(diddoc))
        .route("/.well-known/did/pop.json", get(didpop))
        .with_state(state)
}

async fn diddoc() -> Result<Json<Value>, StatusCode> {
    let storage_dirpath = std::env::var("STORAGE_DIRPATH").map_err(|_| {
        tracing::error!("STORAGE_DIRPATH env variable required");
        StatusCode::NOT_FOUND
    })?;

    match tokio::fs::read_to_string(&format!("{storage_dirpath}/did.json")).await {
        Ok(content) => Ok(Json(serde_json::from_str(&content).unwrap())),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[axum::debug_handler]
async fn didpop(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {
    let challenge = params.get("challenge").ok_or(StatusCode::BAD_REQUEST)?;

    // Load DID document and its verification methods
    let diddoc_value = diddoc().await?.0;
    let diddoc: Document = serde_json::from_value(diddoc_value.clone()).unwrap();

    let did_address = diddoc.id.clone();
    let methods = diddoc.verification_method.clone().unwrap_or(vec![]);

    // Build verifiable credential (VC)
    let vc: VerifiableCredential = serde_json::from_value(json!({
        "@context": DEFAULT_CONTEXT_V2,
        "type": ["VerifiableCredential", "DIDDocument"],
        "issuer": &did_address,
        "validFrom": Utc::now(),
        "credentialSubject": diddoc_value,
    }))
    .unwrap();

    // Embed VC into a verifiable presentation (VP)
    let mut vp: VerifiablePresentation = serde_json::from_value(json!({
        "@context": DEFAULT_CONTEXT_V2,
        "id": format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        "type": ["VerifiablePresentation"],
        "holder": &did_address,
        "verifiableCredential": [vc],
    }))
    .unwrap();

    // Generate proofs of possession
    let mut vec_proof: Vec<Proof> = vec![];

    let options: Proof = serde_json::from_value(json!({
        "type": PROOF_TYPE_DATA_INTEGRITY_PROOF,
        "challenge": challenge,
        "proofPurpose": "",
        "verificationMethod": "",
    }))
    .unwrap();

    for method in methods {
        // Lookup keypair from keystore
        let pubkey = method
            .public_key
            .as_ref()
            .expect("Verification methods should embed public keys.");

        let _ = match pubkey {
            KeyFormat::Jwk(key) => key,
            _ => panic!("Unexpected key format"),
        };

        // Lookup keypair from secret store
        let AppStateRepository {
            secret_repository, ..
        } = state
            .repository
            .as_ref()
            .expect("missing persistence layer");

        let jwk: Jwk = {
            let secret = tokio::runtime::Runtime::new().unwrap().block_on(async {
                secret_repository
                    .find_one_by(doc! { "kid": method.id.clone() })
                    .await
            });

            match secret.expect("Error fetching secret").expect("Missing key").secret_material {
                SecretMaterial::JWK { private_key_jwk: jwk } => {serde_json::from_value(jwk).unwrap()},
                _ => panic!("Unexpected key format"),
            }
        };

        // Amend options for linked data proof with method-specific attributes
        let options = Proof {
            nonce: Some(uuid::Uuid::new_v4().to_string()),
            verification_method: method.id.clone(),
            proof_purpose: {
                let vrel = inspect_vm_relationship(&diddoc, &method.id)
                    .expect("Unsupported verification relationship");

                // Do not provide proofs for key agreement methods
                if vrel == "keyAgreement" {
                    continue;
                }

                vrel
            },
            ..options.clone()
        };

        // Generate proof
        let prover = EdDsaJcs2022 {
            proof: options.clone(),
            key_pair: jwk.try_into().expect("Failure to convert to KeyPair"),
            proof_value_codec: Some(Base::Base58Btc),
        };

        let proof = prover.proof(json!(vp)).expect("Error generating proof");
        vec_proof.push(proof);
    }

    // Insert all proofs
    vp.proof = Some(Proofs::SetOfProofs(vec_proof));

    // Output final verifiable credential
    Ok(Json(json!(vp)))
}

/// Inspects in a DID document the relationship of
/// a verification method based on its identifier
fn inspect_vm_relationship(diddoc: &Document, vm_id: &str) -> Option<String> {
    let vrel = [
        (
            json!(diddoc.authentication.clone().unwrap_or(vec![])),
            String::from("authentication"),
        ),
        (
            json!(diddoc.assertion_method.clone().unwrap_or(vec![])),
            String::from("assertionMethod"),
        ),
        (
            json!(diddoc.key_agreement.clone().unwrap_or(vec![])),
            String::from("keyAgreement"),
        ),
    ];

    for (k, v) in vrel {
        if k.as_array().unwrap().iter().any(|x| {
            let Some(id) = x.as_str() else { return false };
            id == vm_id
        }) {
            return Some(v.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{didgen, util::dotenv_flow_read};
    use shared::utils::tests_utils::tests;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use did_utils::{
        didcore::{Document, KeyFormat, Proofs},
        jwk::Jwk,
        proof::{CryptoProof, EdDsaJcs2022},
        vc::VerifiablePresentation,
    };
    use serde_json::json;
    use tower::util::ServiceExt;

    fn setup_ephemeral_diddoc() -> (String, Document, Arc<AppState>) {
        let storage_dirpath = dotenv_flow_read("STORAGE_DIRPATH")
            .map(|p| format!("{}/{}", p, uuid::Uuid::new_v4()))
            .unwrap();

        let state = tests::setup();

        // Run didgen logic
        let diddoc = didgen::didgen(&storage_dirpath, state.clone()).unwrap();

        // TODO! Find a race-free way to accomodate this. Maybe a test mutex?
        std::env::set_var("STORAGE_DIRPATH", &storage_dirpath);

        (storage_dirpath, diddoc, state)
    }

    fn cleanup(storage_dirpath: &str) {
        std::env::remove_var("STORAGE_DIRPATH");
        std::fs::remove_dir_all(storage_dirpath).unwrap();
    }

    #[tokio::test]
    async fn verify_didpop() {
        // Generate test-restricted did.json
        let (storage_dirpath, expected_diddoc, state) = setup_ephemeral_diddoc();

        let app = routes(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/.well-known/did/pop.json?challenge={}",
                        uuid::Uuid::new_v4()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let vp: VerifiablePresentation = serde_json::from_slice(&body).unwrap();

        let vc = vp.verifiable_credential.get(0).unwrap();
        let diddoc = serde_json::from_value(json!(vc.credential_subject)).unwrap();

        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),
            json_canon::to_string(&expected_diddoc).unwrap()
        );

        let Some(proofs) = &vp.proof else {
            panic!("Verifiable presentation carries no proof")
        };
        let Proofs::SetOfProofs(proofs) = proofs else {
            unreachable!()
        };
        for proof in proofs {
            let pubkey = resolve_vm_for_public_key(&diddoc, &proof.verification_method)
                .expect("ResolutionError");
            let verifier = EdDsaJcs2022 {
                proof: proof.clone(),
                key_pair: pubkey.try_into().expect("Failure to convert to KeyPair"),
                proof_value_codec: None,
            };

            assert!(verifier.verify(json!(vp)).is_ok());
        }

        cleanup(&storage_dirpath);
    }

    fn resolve_vm_for_public_key(diddoc: &Document, vm_id: &str) -> Option<Jwk> {
        let Some(methods) = &diddoc.verification_method else {
            return None;
        };
        let method = methods.iter().find(|m| m.id == vm_id);

        match method {
            None => None,
            Some(m) => {
                let Some(key) = &m.public_key else {
                    return None;
                };
                let KeyFormat::Jwk(jwk) = key else {
                    return None;
                };
                Some(jwk.clone())
            }
        }
    }
}
