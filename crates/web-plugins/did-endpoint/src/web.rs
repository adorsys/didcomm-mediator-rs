use super::plugin::DidEndPointState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use chrono::Utc;

use did_utils::{
    didcore::{Document, KeyFormat, Proofs},
    jwk::Jwk,
    proof::{CryptoProof, EdDsaJcs2022, Proof, PROOF_TYPE_DATA_INTEGRITY_PROOF},
    vc::{VerifiableCredential, VerifiablePresentation},
};
use mongodb::bson::doc;
use multibase::Base;
use serde_json::{json, Value};
use std::{collections::HashMap, convert::TryInto, sync::Arc};
use tokio::{runtime::Handle, task};

const DEFAULT_CONTEXT_V2: &str = "https://www.w3.org/ns/credentials/v2";

pub(crate) fn routes(state: Arc<DidEndPointState>) -> Router {
    Router::new()
        .route("/.well-known/did.json", get(diddoc))
        .route("/.well-known/did/pop.json", get(didpop))
        .with_state(state)
}

pub(crate) async fn diddoc(State(state): State<Arc<DidEndPointState>>) -> impl IntoResponse {
    match state.repository.find_one_by(doc! {}).await {
        Ok(Some(diddoc_entity)) => (StatusCode::OK, Json(diddoc_entity.diddoc)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "DIDDoc not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

#[axum::debug_handler]
async fn didpop(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<DidEndPointState>>,
) -> Result<Json<Value>, StatusCode> {
    let challenge = params.get("challenge").ok_or(StatusCode::BAD_REQUEST)?;

    // Retrieve the DID document from the repository
    let diddoc_entity = state
        .repository
        .find_one_by(doc! {})
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let diddoc_value = diddoc_entity.diddoc;
    let diddoc: Document = serde_json::from_value(json!(diddoc_value))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let did_address = diddoc.id.clone();
    let methods = diddoc.verification_method.clone().unwrap_or_default();

    // Build verifiable credential (VC)
    let vc: VerifiableCredential = serde_json::from_value(json!({
        "@context": DEFAULT_CONTEXT_V2,
        "type": ["VerifiableCredential", "DIDDocument"],
        "issuer": &did_address,
        "validFrom": Utc::now(),
        "credentialSubject": diddoc_value,
    }))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Embed VC into a verifiable presentation (VP)
    let mut vp: VerifiablePresentation = serde_json::from_value(json!({
        "@context": DEFAULT_CONTEXT_V2,
        "id": format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        "type": ["VerifiablePresentation"],
        "holder": &did_address,
        "verifiableCredential": [vc],
    }))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate proofs of possession
    let mut vec_proof: Vec<Proof> = vec![];

    let options: Proof = serde_json::from_value(json!({
        "type": PROOF_TYPE_DATA_INTEGRITY_PROOF,
        "challenge": challenge,
        "proofPurpose": "",
        "verificationMethod": "",
    }))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let keystore = state.keystore.clone();

    for method in methods {
        // Lookup keypair from keystore
        let pubkey = method
            .public_key
            .as_ref()
            .expect("Verification methods should embed public keys.");

        let kid = crate::util::handle_vm_id(&method.id, &diddoc);

        let jwk: Jwk = match pubkey {
            KeyFormat::Jwk(_) => task::block_in_place(|| {
                Handle::current().block_on(async {
                    keystore
                        .retrieve(&kid)
                        .await
                        .expect("Error fetching secret")
                        .expect("Secret not found")
                })
            }),
            _ => panic!("Unsupported key format"),
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
            json!(diddoc.authentication.clone().unwrap_or_default()),
            String::from("authentication"),
        ),
        (
            json!(diddoc.assertion_method.clone().unwrap_or_default()),
            String::from("assertionMethod"),
        ),
        (
            json!(diddoc.key_agreement.clone().unwrap_or_default()),
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
    use crate::{
        didgen::tests::*,
        persistence::{tests::MockDidDocumentRepository, MediatorDidDocument},
    };

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use database::Repository;
    use did_utils::{
        didcore::{Document, KeyFormat, Proofs},
        jwk::Jwk,
        proof::{CryptoProof, EdDsaJcs2022},
        vc::VerifiablePresentation,
    };
    use http_body_util::BodyExt;
    use keystore::Keystore;
    use serde_json::json;
    use tower::util::ServiceExt;

    #[tokio::test(flavor = "multi_thread")]
    async fn verify_didpop() {
        dotenv_flow::from_filename("../../../.env.example").ok();

        let expected_diddoc: Document = serde_json::from_str(
            r##"{
                "@context": ["https://www.w3.org/ns/did/v1"],
                "id": "did:peer:123",
                "verificationMethod": [
                    {
                        "id": "#key-1",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:123",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": "PuG2L5um-tAnHlvT29gTm9Wj9fZca16vfBCPKsHB5cA"
                        }
                    }
                ],
                "authentication": ["#key-1"]
            }"##,
        )
        .unwrap();

        let kid = "did:peer:123#key-1";
        let repository = MockDidDocumentRepository::new();
        let mock_keystore = Keystore::with_mock_configs(vec![(kid.to_string(), setup())]);

        repository
            .store(MediatorDidDocument {
                id: None,
                diddoc: expected_diddoc.clone(),
            })
            .await
            .unwrap();

        // Setup state with mocks
        let state = DidEndPointState {
            keystore: mock_keystore,
            repository: Arc::new(repository),
        };

        let app = routes(Arc::new(state));
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

        let body = BodyExt::collect(response.into_body()).await.unwrap();
        let vp: VerifiablePresentation = serde_json::from_slice(&body.to_bytes()).unwrap();

        let vc = vp.verifiable_credential.first();
        let diddoc = serde_json::from_value(json!(vc.unwrap().credential_subject)).unwrap();

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
                Some(*jwk.clone())
            }
        }
    }
}
