//! This module provides utilities for creating and verifying proofs.
//! 
//! See [RFC 7807](https://tools.ietf.org/html/rfc7807) for details.
//! 
//! # Examples
//! 
//! ```no run
//! # use chrono::TimeZone;
//! # use serde_json::json;
//! 
//! # use crate::crypto::Ed25519KeyPair;
//! 
//! # use crate::*;
//! 
//! let my_string = String::from("Sample seed bytes of thirtytwo!b");
//! let seed: &[u8] = my_string.as_bytes();
//! let key_pair = Ed25519KeyPair::new_with_seed(seed)?;
//! let public_key = &key_pair.public_key.clone();
//! 
//! let proof = Proof {
//!     id: None,
//!     proof_type: "DataIntegrityProof".to_string(),
//!     cryptosuite: Some("jcs-eddsa-2022".to_string()),
//!     proof_purpose: "assertionMethod".to_string(),
//!     verification_method: "https://di.example/issuer#z6MkjLrk3gKS2nnkeWcmcxiZPGskmesDpuwRBorgHxUXfxnG".to_string(),
//!     created: Some(chrono::Utc.with_ymd_and_hms(2023, 3, 5, 19, 23, 24).unwrap()),
//!     expires: None,
//!     domain: Some(crate::proof::model::Domain::SingleString("vc-demo.adorsys.com".to_string())),
//!     challenge: Some("523452345234asfdasdfasdfa".to_string()),
//!     proof_value: None,
//!     previous_proof: None,
//!     nonce: Some("1234567890".to_string()),
//! };
//! 
//! let payload = json!({
//!     "id": "did:example:123456789abcdefghi",
//!     "name": "Alice",
//!     "age": 101,
//!     "image": "data:image/png;base64,iVBORw0KGgo...kJggg==",
//! });
//! 
//! let ed_dsa_jcs_2022_prover = EdDsaJcs2022 {
//!     proof,
//!     key_pair,
//!     proof_value_codec: Some(Base::Base58Btc),
//! };
//! 
//! let secured_proof = ed_dsa_jcs_2022_prover.proof(payload.clone())?;
//! 
//! let secure_doc = UnsecuredDocument {
//!     content: payload,
//!     proof: crate::proof::model::Proofs::SingleProof(Box::new(secured_proof.clone())),
//! };
//! 
//! // Serialize the struct into a serde_json::Value
//! let secure_doc_json_value: Value = serde_json::to_value(&secure_doc)?;
//! 
//! let ed_dsa_jcs_2022_verifier = EdDsaJcs2022 {
//!     proof: secured_proof,
//!     key_pair: Ed25519KeyPair::from_public_key(public_key.as_bytes())?,
//!     proof_value_codec: None,
//! };
//! 
//! // Verify the proof
//! ed_dsa_jcs_2022_verifier.verify(secure_doc_json_value)?;
//!```
mod model;
mod traits;
mod eddsa_jcs_2022;

// public re-exports
pub use eddsa_jcs_2022::{EdDsaJcs2022, CRYPRO_SUITE_EDDSA_JCS_2022, PROOF_TYPE_DATA_INTEGRITY_PROOF};
pub use model::{Proof, UnsecuredDocument, Domain, PreviousProofs, Proofs};
pub use traits::CryptoProof;