use multibase::Base;

use crate::crypto::{
    ed25519::Ed25519KeyPair,
    sha256_hash::sha256_hash,
    traits::{CoreSign, Error},
};

use super::{model::Proof, traits::CryptoProof};

pub const CRYPRO_SUITE_EDDSA_JCS_2022: &str = "eddsa-jcs-2022";
pub const PROOF_TYPE_DATA_INTEGRITY_PROOF: &str = "DataIntegrityProof";

pub struct EdDsaJcs2022 {
    /// The proof object
    ///
    /// In a proof creation process, it does not contain the proof value, but
    ///   carries info like challenge, nonce, etc.
    ///
    /// In a proof verification process, it contains the proof as found in the
    ///   secured document, including the proof value
    pub proof: Proof,

    /// The keypair used to sreate the proof: in which case the signing key must be present.
    ///
    /// The keypair used to verify the proof: in which case only the public key must be present.
    ///
    /// This module does not perform resolution of the verification method. Module assumes calles
    /// extracted the public key prior to calling this module.
    pub key_pair: Ed25519KeyPair,

    /// The proof value codec. This is important for the encoding of the proof.
    ///
    /// For the decoding, codec is automaticaly infered from the string.
    pub proof_value_codec: Option<Base>,
}

impl CryptoProof for EdDsaJcs2022 {
    fn proof(&self, payload: serde_json::Value) -> Result<Proof, Error> {
        match self.proof_value_codec {
            None => Err(Error::InvalidCall("proof_value_codec must be set for proof creation".to_string())),
            Some(_) => {
                let normalized_proof = Proof {
                    proof_type: PROOF_TYPE_DATA_INTEGRITY_PROOF.to_string(),
                    cryptosuite: Some(CRYPRO_SUITE_EDDSA_JCS_2022.to_string()),
                    created: match self.proof.created {
                        Some(created) => Some(created),
                        None => Some(chrono::Utc::now()),
                    },
                    proof_value: None,
                    ..self.proof.clone()
                };

                // Canonicalization
                let canon_proof = json_canon::to_string(&normalized_proof).map_err(|_| Error::InvalidProof)?;
                let canon_doc = json_canon::to_string(&payload).map_err(|_| Error::InvalidProof)?;

                // Compute hash to sign
                let hash = [sha256_hash(canon_proof.as_bytes()), sha256_hash(canon_doc.as_bytes())].concat();

                return self.key_pair.sign(&hash[..]).map(|signature| Proof {
                    proof_value: Some(multibase::encode(self.proof_value_codec.unwrap(), signature)),
                    ..normalized_proof
                });
            }
        }
    }

    fn verify(&self, payload: serde_json::Value) -> Result<(), Error> {
        match self.proof.proof_value.clone() {
            None => Err(Error::InvalidProof),
            Some(proof_value) => {
                // Clone the proof
                // - droping the proof value
                // - normalyzing proof type fields
                // This is the document to be signed
                let normalized_proof = Proof {
                    proof_value: None,
                    proof_type: PROOF_TYPE_DATA_INTEGRITY_PROOF.to_string(),
                    cryptosuite: Some(CRYPRO_SUITE_EDDSA_JCS_2022.to_string()),
                    ..self.proof.clone()
                };

                // Strip the proof from the payload if any.
                let naked_payload = match payload.get("proof") {
                    None => payload,
                    Some(_) => {
                        let mut naked_payload = payload.clone();
                        naked_payload.as_object_mut().unwrap().remove("proof");
                        naked_payload
                    }
                };

                // Canonicalization
                let canon_proof = json_canon::to_string(&normalized_proof).map_err(|_| Error::InvalidProof)?;
                let canon_doc = json_canon::to_string(&naked_payload).map_err(|_| Error::InvalidProof)?;

                // Compute hash to verify
                let hash = [sha256_hash(canon_proof.as_bytes()), sha256_hash(canon_doc.as_bytes())].concat();

                return multibase::decode(proof_value)
                    .map_err(|_| Error::InvalidProof)
                    .and_then(|signature| self.key_pair.verify(&hash, &(signature.1)));
            }
        }
    }
}

// the test module
#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::{crypto::traits::Generate, proof::model::UnsecuredDocument};

    // create an EdDsaJcs2022 object and use it to produce a proof.
    // The proof is then verified.
    #[test]
    fn test_create_verify_proof() {
        use chrono::TimeZone;
        use serde_json::json;

        use crate::crypto::ed25519::Ed25519KeyPair;

        use super::*;

        let my_string = String::from("Sample seed bytes of thirtytwo!b");
        let seed: &[u8] = my_string.as_bytes();
        let key_pair = Ed25519KeyPair::new_with_seed(seed).unwrap();
        let public_key = &key_pair.public_key.clone();

        let proof = Proof {
            id: None,
            proof_type: "DataIntegrityProof".to_string(),
            cryptosuite: Some("jcs-eddsa-2022".to_string()),
            proof_purpose: "assertionMethod".to_string(),
            verification_method: "https://di.example/issuer#z6MkjLrk3gKS2nnkeWcmcxiZPGskmesDpuwRBorgHxUXfxnG".to_string(),
            created: Some(chrono::Utc.with_ymd_and_hms(2023, 3, 5, 19, 23, 24).unwrap()),
            expires: None,
            domain: Some(crate::proof::model::Domain::SingleString("vc-demo.adorsys.com".to_string())),
            challenge: Some("523452345234asfdasdfasdfa".to_string()),
            proof_value: None,
            previous_proof: None,
            nonce: Some("1234567890".to_string()),
        };

        let payload = json!({
            "id": "did:example:123456789abcdefghi",
            "name": "Alice",
            "age": 101,
            "image": "data:image/png;base64,iVBORw0KGgo...kJggg==",
        });

        let ed_dsa_jcs_2022_prover = EdDsaJcs2022 {
            proof,
            key_pair,
            proof_value_codec: Some(Base::Base58Btc),
        };

        let secured_proof = ed_dsa_jcs_2022_prover.proof(payload.clone()).unwrap();

        let secure_doc = UnsecuredDocument {
            content: payload,
            proof: crate::proof::model::Proofs::SingleProof(Box::new(secured_proof.clone())),
        };

        let expected_canonicalized_proof = r#"{"challenge":"523452345234asfdasdfasdfa","created":"2023-03-05T19:23:24Z","cryptosuite":"eddsa-jcs-2022","domain":"vc-demo.adorsys.com","nonce":"1234567890","proofPurpose":"assertionMethod","proofValue":"z2DbDNkE47SquDQ7wM6p3RjNdFB1FG7Num2w9kprZjUB2gNZvz7bYgcT5XCe3TdjfxxWfKkup1ZdrRhfEMLsk2kmr","type":"DataIntegrityProof","verificationMethod":"https://di.example/issuer#z6MkjLrk3gKS2nnkeWcmcxiZPGskmesDpuwRBorgHxUXfxnG"}"#;
        let canonicalized_proof = json_canon::to_string(&secured_proof).unwrap();
        assert_eq!(expected_canonicalized_proof, canonicalized_proof);

        // let canonicalized_secured_doc = json_canon::to_string(&secure_doc).unwrap();
        // Serialize the struct into a serde_json::Value
        let secure_doc_json_value: Value = serde_json::to_value(&secure_doc).expect("Failed to serialize");

        let ed_dsa_jcs_2022_verifier = EdDsaJcs2022 {
            proof: secured_proof,
            key_pair: Ed25519KeyPair::from_public_key(public_key.as_bytes()).unwrap(),
            proof_value_codec: None,
        };

        ed_dsa_jcs_2022_verifier.verify(secure_doc_json_value).unwrap();
    }
}
