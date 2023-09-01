use std::collections::HashMap;

use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::proof::model::Proof;

// === Structure of a did document ===

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    // The @context property defines the vocabulary used in the JSON-LD document.
    // It provides a way to map the keys in the JSON structure to specific terms,
    // properties, and classes from external vocabularies. In the context of a
    // DID Document, the @context property is used to define the vocabulary for
    // the various properties within the document, such as id, publicKey, service, and others.
    #[serde(rename = "@context")]
    pub context: Context,

    // === Identifier ===

    // Identifier property is mandatory in a did document.
    // see https://www.w3.org/TR/did-core/#dfn-id
    pub id: String,

    // See https://www.w3.org/TR/did-core/#dfn-controller
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<Controller>,

    // See https://www.w3.org/TR/did-core/#dfn-alsoknownas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<Vec<String>>,

    // === Verification Methods ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_method: Option<Vec<VerificationMethod>>,

    // === Verification Relationships ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Vec<Authentication>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertion_method: Option<Vec<AssertionMethod>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_delegation: Option<Vec<CapabilityDelegation>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_invocation: Option<Vec<CapabilityInvocation>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_agreement: Option<Vec<KeyAgreement>>,

    // === Services ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Vec<Service>>,

    // === Dynamic Properties ===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,

    // === Proof ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<Proofs>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Context {
    SingleString(String),
    SetOfString(Vec<String>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Controller {
    SingleString(String),
    SetOfString(Vec<String>),
}

// See https://www.w3.org/TR/did-core/#services
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,

    #[serde(rename = "type")]
    pub service_type: String,

    pub service_endpoint: String,

    // === Additional properties ===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Default, Deserialize)]
pub struct VerificationMethod {
    pub id: String,

    #[serde(rename = "type")]
    pub key_type: String,

    pub controller: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "publicKeyBase58")]
    #[serde(alias = "publicKeyMultibase")]
    #[serde(alias = "publicKeyJwk")]
    #[serde(serialize_with = "VerificationMethod::serialize_public_key_format")]
    #[serde(flatten)]
    pub public_key: Option<KeyFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "privateKeyBase58")]
    #[serde(alias = "privateKeyMultibase")]
    #[serde(alias = "privateKeyJwk")]
    #[serde(serialize_with = "VerificationMethod::serialize_private_key_format")]
    #[serde(flatten)]
    pub private_key: Option<KeyFormat>,

    // === Additional properties ===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum KeyFormat {
    Base58(String),
    Multibase(String),
    Jwk(Jwk),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct Jwk {
    #[serde(rename = "kid", skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    #[serde(rename = "kty")]
    pub key_type: String,
    #[serde(rename = "crv")]
    pub curve: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<String>,
}

// === Authentication ===
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Authentication {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

// === Assertion Method ===
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum AssertionMethod {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

// === Capability Delegation ===
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CapabilityDelegation {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

// === Capability Invocation ===
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CapabilityInvocation {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

// === Key Agreement ===
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum KeyAgreement {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

impl VerificationMethod {
    fn serialize_private_key_format<S>(value: &Option<KeyFormat>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(KeyFormat::Base58(s)) => {
                let mut obj = serializer.serialize_map(Some(1))?;
                obj.serialize_entry("privateKeyBase58", s)?;
                obj.end()
            }
            Some(KeyFormat::Multibase(s)) => {
                let mut obj = serializer.serialize_map(Some(1))?;
                obj.serialize_entry("privateKeyMultibase", s)?;
                obj.end()
            }
            Some(KeyFormat::Jwk(jwk)) => {
                let mut obj = serializer.serialize_map(Some(1))?;
                obj.serialize_entry("privateKeyJwk", jwk)?;
                obj.end()
            }
            None => serializer.serialize_none(),
        }
    }

    fn serialize_public_key_format<S>(value: &Option<KeyFormat>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(KeyFormat::Base58(s)) => {
                let mut obj = serializer.serialize_map(Some(1))?;
                obj.serialize_entry("publicKeyBase58", s)?;
                obj.end()
            }
            Some(KeyFormat::Multibase(s)) => {
                let mut obj = serializer.serialize_map(Some(1))?;
                obj.serialize_entry("publicKeyMultibase", s)?;
                obj.end()
            }
            Some(KeyFormat::Jwk(jwk)) => {
                let mut obj = serializer.serialize_map(Some(1))?;
                obj.serialize_entry("publicKeyJwk", jwk)?;
                obj.end()
            }
            None => serializer.serialize_none(),
        }
    }
}

// === Proof ===
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Proofs {
    SingleProof(Box<Proof>),
    SetOfProofs(Vec<Proof>),
}


#[cfg(test)]
pub mod tests {

    // A test that reads the file at ../test_resources/did_example_1.json, uses serde_json to convert
    // the content into a json string uses the json_canon library to canonicalize the json string.
    // The canonicalized json string is then compared to the expected canonicalized json string from the
    // file ../test_resources/did_example_1_canonicalized.json.
    // This example also test parsing and writing a KeyFormat in a verification method.
    #[test]
    fn test_canonicalize_didcore_example_01() {
        read_write_did(
            "test_resources/didcore_example_01.json",
            "test_resources/didcore_example_01_canonicalized.json",
        )
        .unwrap();
    }

    // An example of a relative DID URL used to reference a verification method in a DID document.
    // See https://www.w3.org/TR/did-core/#example-an-example-of-a-relative-did-url
    #[test]
    fn test_canonicalize_didcore_example_09() {
        read_write_did(
            "test_resources/didcore_example_09.json",
            "test_resources/didcore_example_09_canonicalized.json",
        )
        .unwrap();
    }

    // DID document with a controller property
    // The context entry in this case is not a list but a single string value.
    // The json document is non conform, has it is carying a trailing comma. Must be cleaned before processing.
    // See https://www.w3.org/TR/did-core/#example-did-document-with-a-controller-property
    #[test]
    fn test_canonicalize_didcore_example_11() {
        read_write_did(
            "test_resources/didcore_example_11.json",
            "test_resources/didcore_example_11_canonicalized.json",
        )
        .unwrap();
    }

    // Verification methods using publicKeyJwk and publicKeyMultibase
    // TODO: verify that the same key is not used in two verification methods.!!!
    // As, A verification method MUST NOT contain multiple verification material properties for the same material.
    // TODO: verify that the types are supported verification methods.
    // See https://www.w3.org/TR/did-core/#example-various-verification-method-types
    #[test]
    fn test_canonicalize_didcore_example_13() {
        read_write_did(
            "test_resources/didcore_example_13.json",
            "test_resources/didcore_example_13_canonicalized.json",
        )
        .unwrap();
    }

    // Embedding and referencing verification methods
    // Authentication property containing three verification methods (covering example 15)
    // Tests that the authentication entries can be read and written.
    // TODO: verify that the referecing verification relationship correspond to a verification method in the document.
    // TODO: verify that the types are supported verification methods for authentication.
    // See https://www.w3.org/TR/did-core/#example-embedding-and-referencing-verification-methods
    // See https://www.w3.org/TR/did-core/#example-authentication-property-containing-three-verification-methods
    #[test]
    fn test_canonicalize_didcore_example_14() {
        read_write_did(
            "test_resources/didcore_example_14.json",
            "test_resources/didcore_example_14_canonicalized.json",
        )
        .unwrap();
    }

    // Assertion method property containing two verification methods
    // Tests that the assertionMethod entries can be read and written.
    // TODO: verify that the referecing verification relationship correspond to a verification method in the document.
    // TODO: verify that the types are supported verification methods for assertionMethod.
    // See https://www.w3.org/TR/did-core/#example-assertion-method-property-containing-two-verification-methods
    #[test]
    fn test_canonicalize_didcore_example_16() {
        read_write_did(
            "test_resources/didcore_example_16.json",
            "test_resources/didcore_example_16_canonicalized.json",
        )
        .unwrap();
    }

    // Key agreement property containing two verification methods
    // Tests that the keyAgreement entries can be read and written.
    // TODO: verify that the referecing verification relationship correspond to a verification method in the document.
    // TODO: verify that the types are supported verification methods for keyAgreement (ECDH).
    // See https://www.w3.org/TR/did-core/#example-key-agreement-property-containing-two-verification-methods
    #[test]
    fn test_canonicalize_didcore_example_17() {
        read_write_did(
            "test_resources/didcore_example_17.json",
            "test_resources/didcore_example_17_canonicalized.json",
        )
        .unwrap();
    }

    // Capability invocation property containing two verification methods
    // Tests that the capabilityInvocation entries can be read and written.
    // TODO: verify that the referecing verification relationship correspond to a verification method in the document.
    // TODO: verify that the types are supported verification methods for capabilityInvocation.
    // See https://www.w3.org/TR/did-core/#example-capability-invocation-property-containing-two-verification-methods
    #[test]
    fn test_canonicalize_didcore_example_18() {
        read_write_did(
            "test_resources/didcore_example_18.json",
            "test_resources/didcore_example_18_canonicalized.json",
        )
        .unwrap();
    }

    // Capability delegation property containing two verification methods
    // Tests that the capabilityDelegation entries can be read and written.
    // TODO: verify that the referecing verification relationship correspond to a verification method in the document.
    // TODO: verify that the types are supported verification methods for capabilityDelegation.
    // See https://www.w3.org/TR/did-core/#example-capability-delegation-property-containing-two-verification-methods
    #[test]
    fn test_canonicalize_didcore_example_19() {
        read_write_did(
            "test_resources/didcore_example_19.json",
            "test_resources/didcore_example_19_canonicalized.json",
        )
        .unwrap();
    }

    // Usage of the service property
    // Tests that the service entries can be read and written.
    // See https://www.w3.org/TR/did-core/#example-usage-of-the-service-property
    #[test]
    fn test_canonicalize_didcore_example_20() {
        read_write_did(
            "test_resources/didcore_example_20.json",
            "test_resources/didcore_example_20_canonicalized.json",
        )
        .unwrap();
    }

    // read a file given the path as method param and write content to console
    // std::fs::read_to_string() expects the path from the project root.
    fn read_write_did(raw_path: &str, canon_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let did_document = std::fs::read_to_string(raw_path)?;
        let document: super::Document = serde_json::from_str(&did_document).unwrap();

        let canonicalized = json_canon::to_string(&document).unwrap();

        let expected = std::fs::read_to_string(canon_path)?;
        assert_eq!(expected, canonicalized);

        Ok(())
    }
}
