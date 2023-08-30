use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Proof {

    // An optional identifier for the proof.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    // A specified set of cryptographic primitives bundled together into a cryptographic suite
    // See https://www.w3.org/TR/vc-data-integrity/#dfn-proof-type
    #[serde(rename = "type")]
    pub proof_type: String,


    // See https://www.w3.org/TR/vc-data-integrity/#dfn-proof-purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_purpose: Option<String>,

    // See https://www.w3.org/TR/vc-data-integrity/#dfn-verification-method
    pub verification_method: String,

    // The date and time the proof was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,

    // The date and time that the proof expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<DateTime<Utc>>,

    // One or more security domains in which the proof is meant to be used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Domain>,

    // A string value that SHOULD be included in a proof if a domain is specified
    // The value is used once for a particular domain and window of time
    // This value is used to mitigate replay attacks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,

    // Data necessary to verify the digital proof using the verificationMethod specified
    // The contents of the value MUST be a [MULTIBASE]-encoded binary value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_value: Option<String>,

    // Each value identifies another data integrity proof that 
    // MUST verify before the current proof is processed
    // See https://www.w3.org/TR/vc-data-integrity/#proof-chains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_proof: Option<PreviousProof>,

    // A string value supplied by the proof creator that is unique to the proof
    // One use of this field is to increase privacy by decreasing linkability 
    // that is the result of deterministically generated signatures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Domain {
    SingleString(String),
    SetOfString(Vec<String>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PreviousProof {
    SingleString(String),
    SetOfString(Vec<String>),
}
