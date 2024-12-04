use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Represents the cryptographic proof of a verifiable credential
pub struct Proof {
    /// An optional identifier for the proof.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// A specified set of cryptographic primitives bundled together into a cryptographic suite.
    /// See [proof type]
    ///
    /// [proof type]: https://www.w3.org/TR/vc-data-integrity/#dfn-proof-type
    #[serde(rename = "type")]
    pub proof_type: String,

    /// A string value that identifies the cryptographic suite used to create the proof.
    /// Only required when type=DataIntegrityProof
    pub cryptosuite: Option<String>,

    /// The [purpose] of the proof.
    ///
    /// [purpose]: https://www.w3.org/TR/vc-data-integrity/#dfn-proof-purpose
    pub proof_purpose: String,

    /// A set of parameters that can be used together with a process to independently verify a proof.
    /// See [verification method][vm]
    ///
    /// [vm]: https://www.w3.org/TR/vc-data-integrity/#dfn-verification-method
    pub verification_method: String,

    /// The date and time the proof was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,

    /// The date and time that the proof expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<DateTime<Utc>>,

    /// One or more security domains in which the proof is meant to be used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Domain>,

    /// A string value that SHOULD be included in a proof if a domain is specified.
    /// The value is used once for a particular domain and window of time.
    /// This value is used to mitigate replay attacks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,

    /// Data necessary to verify the digital proof using the verificationMethod specified
    /// The contents of the value MUST be a multibase-encoded binary value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_value: Option<String>,

    /// Each value identifies another data integrity proof that
    /// MUST verify before the current proof is processed
    // See https://www.w3.org/TR/vc-data-integrity/#proof-chains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_proof: Option<PreviousProofs>,

    /// A string value supplied by the proof creator that is unique to the proof.
    /// One use of this field is to increase privacy by decreasing linkability
    /// that is the result of deterministically generated signatures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
/// The domain in which the proof is meant to be used
pub enum Domain {
    SingleString(String),
    SetOfString(Vec<String>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
/// The previous proofs in the proof chain
pub enum PreviousProofs {
    SingleString(String),
    SetOfString(Vec<String>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
/// The set of proofs
pub enum Proofs {
    SingleProof(Box<Proof>),
    SetOfProofs(Box<Vec<Proof>>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// The unsecured document
pub struct UnsecuredDocument {
    /// The document to be secured
    #[serde(flatten)]
    pub content: Value,

    /// Set of proofs
    pub proof: Proofs,
}

// the test module
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    // creates a proof object that serializes to give expected tring
    #[test]
    fn test_create_serialize_proof() {
        let proof = Proof {
            id: None,
            proof_type: "DataIntegrityProof".to_string(),
            cryptosuite: Some("jcs-eddsa-2022".to_string()),
            proof_purpose: "assertionMethod".to_string(),
            verification_method: "https://di.example/issuer#z6MkjLrk3gKS2nnkeWcmcxiZPGskmesDpuwRBorgHxUXfxnG".to_string(),
            created: Some(Utc.with_ymd_and_hms(2023, 3, 5, 19, 23, 24).unwrap()),
            expires: None,
            domain: None,
            challenge: None,
            proof_value: Some("zQeVbY4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYV8nQApmEcqaqA3Q1gVHMrXFkXJeV6doDwLWx".to_string()),
            previous_proof: None,
            nonce: None,
        };

        let canonicalized_actual = json_canon::to_string(&proof).unwrap();

        let canonicalized_expected = r#"{"created":"2023-03-05T19:23:24Z","cryptosuite":"jcs-eddsa-2022","proofPurpose":"assertionMethod","proofValue":"zQeVbY4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYV8nQApmEcqaqA3Q1gVHMrXFkXJeV6doDwLWx","type":"DataIntegrityProof","verificationMethod":"https://di.example/issuer#z6MkjLrk3gKS2nnkeWcmcxiZPGskmesDpuwRBorgHxUXfxnG"}"#;

        assert_eq!(canonicalized_expected, canonicalized_actual);
    }

    // Add a single proof to a josn object
    #[test]
    fn test_add_proof_to_unsecure_document() {
        let doc_json = r#"{
            "@context": [
                {"title": "https://schema.org/title"},
                "https://w3id.org/security/data-integrity/v1"
            ],
            "title": "Hello world!"
        }"#;

        let proof_json = r#"{
            "type": "DataIntegrityProof",
            "cryptosuite": "ecdsa-2019",
            "created": "2020-06-11T19:14:04Z",
            "verificationMethod": "https://ldi.example/issuer#zDnaepBuvsQ8cpsWrVKw8fbpGpvPeNSjVPTWoq6cRqaYzBKVP",
            "proofPurpose": "assertionMethod"
        }"#;

        // parse the document into a json value object
        let doc: Value = serde_json::from_str(doc_json).unwrap();
        // parse tthe proof into a proof object
        let proof: Proof = serde_json::from_str(proof_json).unwrap();
        let canonicalized_expected = r#"{"@context":[{"title":"https://schema.org/title"},"https://w3id.org/security/data-integrity/v1"],"proof":{"created":"2020-06-11T19:14:04Z","cryptosuite":"ecdsa-2019","proofPurpose":"assertionMethod","type":"DataIntegrityProof","verificationMethod":"https://ldi.example/issuer#zDnaepBuvsQ8cpsWrVKw8fbpGpvPeNSjVPTWoq6cRqaYzBKVP"},"title":"Hello world!"}"#;
        let proofs = Proofs::SingleProof(Box::new(proof));

        generic_add_proof_to_document(doc, proofs, canonicalized_expected).unwrap();
    }

    // Add a list of proof to the unsecure object
    #[test]
    fn test_add_proofs_to_secured_document() {
        let doc_json = r#"{
            "@context": [
                {"title": "https://schema.org/title"},
                "https://w3id.org/security/data-integrity/v1"
            ],
            "title": "Hello world!"
        }"#;

        let proof_json = r#"[{
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-2022",
            "created": "2020-11-05T19:23:24Z",
            "verificationMethod": "https://ldi.example/issuer/1#z6MkjLrk3gKS2nnkeWcmcxiZPGskmesDpuwRBorgHxUXfxnG",
            "proofPurpose": "assertionMethod",
            "proofValue": "z4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYVQeVbY8nQAVHMrXFkXJpmEcqdoDwLWxaqA3Q1geV6"
        }, {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-2022",
            "created": "2020-11-05T13:08:49Z",
            "verificationMethod": "https://pfps.example/issuer/2#z6MkGskxnGjLrk3gKS2mesDpuwRBokeWcmrgHxUXfnncxiZP",
            "proofPurpose": "assertionMethod",
            "proofValue": "z5QLBrp19KiWXerb8ByPnAZ9wujVFN8PDsxxXeMoyvDqhZ6Qnzr5CG9876zNht8BpStWi8H2Mi7XCY3inbLrZrm95"
        }]"#;

        // parse the document into a json value object
        let doc: Value = serde_json::from_str(doc_json).unwrap();
        // parse tthe proof into a proof object
        let proof_list: Vec<Proof> = serde_json::from_str(proof_json).unwrap();
        let canonicalized_expected = r#"{"@context":[{"title":"https://schema.org/title"},"https://w3id.org/security/data-integrity/v1"],"proof":[{"created":"2020-11-05T19:23:24Z","cryptosuite":"eddsa-2022","proofPurpose":"assertionMethod","proofValue":"z4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYVQeVbY8nQAVHMrXFkXJpmEcqdoDwLWxaqA3Q1geV6","type":"DataIntegrityProof","verificationMethod":"https://ldi.example/issuer/1#z6MkjLrk3gKS2nnkeWcmcxiZPGskmesDpuwRBorgHxUXfxnG"},{"created":"2020-11-05T13:08:49Z","cryptosuite":"eddsa-2022","proofPurpose":"assertionMethod","proofValue":"z5QLBrp19KiWXerb8ByPnAZ9wujVFN8PDsxxXeMoyvDqhZ6Qnzr5CG9876zNht8BpStWi8H2Mi7XCY3inbLrZrm95","type":"DataIntegrityProof","verificationMethod":"https://pfps.example/issuer/2#z6MkGskxnGjLrk3gKS2mesDpuwRBokeWcmrgHxUXfnncxiZP"}],"title":"Hello world!"}"#;
        let proofs = Proofs::SetOfProofs(Box::new(proof_list));

        generic_add_proof_to_document(doc, proofs, canonicalized_expected).unwrap();
    }

    fn generic_add_proof_to_document(doc: Value, proofs: Proofs, canonicalized_expected: &str) -> Result<(), Box<dyn std::error::Error>> {
        // create the unsecure document
        let unsecure_doc = UnsecuredDocument { content: doc, proof: proofs };

        // serialize the unsecure document
        let canonicalized_actual = json_canon::to_string(&unsecure_doc).unwrap();

        assert_eq!(canonicalized_expected, canonicalized_actual);

        Ok(())
    }
}
