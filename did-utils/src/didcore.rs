//! Implements the DID Core specification
//! 
//! As specified by [Decentralized Identifiers (DIDs) v1.0 - Core architecture,
//! data model, and representations][did-core].
//!
//! [did-core]: https://www.w3.org/TR/did-core/

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{key_jwk::Jwk, ldmodel::Context, proof::Proof};

// === Structure of a did document ===

/// Represents a DID Document according to the [DID Core specification][did-core].
/// 
/// [did-core]: https://www.w3.org/TR/did-core/
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
    #[serde(default = "String::new")]
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

impl Default for Document {  
    fn default() -> Self {  
        let id = String::new();  
        let context = Context::SingleString(String::from("https://www.w3.org/ns/did/v1"));  
        
        Self::new(context, id)  
    }  
}

/// Represents a DID Document controller(s).
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Controller {
    SingleString(String),
    SetOfString(Vec<String>),
}

/// Represents a [service] in a DID Document.
/// 
/// A service defines how to interact with the DID subject.
/// 
/// [service]: https://www.w3.org/TR/did-core/#services
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    #[serde(default = "String::new")]
    pub id: String,

    #[serde(rename = "type")]
    pub service_type: String,

    pub service_endpoint: String,

    // === Additional properties ===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Represents a [verification method] in a DID Document.
/// 
/// [verification method]: https://www.w3.org/TR/did-core/#verification-methods
#[derive(Serialize, Debug, Clone, PartialEq, Default, Deserialize)]
pub struct VerificationMethod {
    pub id: String,

    #[serde(rename = "type")]
    pub key_type: String,

    pub controller: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "VerificationMethod::serialize_public_key_format")]
    #[serde(deserialize_with = "VerificationMethod::deserialize_public_key_format")]
    #[serde(flatten)]
    pub public_key: Option<KeyFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "VerificationMethod::serialize_private_key_format")]
    #[serde(deserialize_with = "VerificationMethod::deserialize_private_key_format")]
    #[serde(flatten)]
    pub private_key: Option<KeyFormat>,

    // === Additional properties ===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Represents different formats of keys used in verification methods.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum KeyFormat {
    Base58(String),
    Multibase(String),
    Jwk(Jwk),
}

/// Represents the authentication methods in a DID Document.
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Authentication {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

/// Represents the assertion methods in a DID Document.
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum AssertionMethod {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

/// Represents the capability delegation methods in a DID Document.
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CapabilityDelegation {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

/// Represents the capability invocation methods in a DID Document.
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CapabilityInvocation {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

/// Represents the key agreement methods in a DID Document.
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum KeyAgreement {
    Reference(String),
    Embedded(Box<VerificationMethod>),
}

impl VerificationMethod {

    /// Serializes the private key format into a JSON map with the appropriate key format field.
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

    /// Serializes the public key format into a JSON map with the appropriate key format field.
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

    /// Deserializes the private key format from a JSON map with the appropriate key format field.
    pub fn deserialize_public_key_format<'de, D>(deserializer: D) -> Result<Option<KeyFormat>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct PublicKeyFormat {
            public_key_base58: Option<String>,
            public_key_multibase: Option<String>,
            public_key_jwk: Option<Jwk>,
        }

        let s: PublicKeyFormat = PublicKeyFormat::deserialize(deserializer)?;

        if s.public_key_base58.is_some() {
            return Ok(Some(KeyFormat::Base58(s.public_key_base58.unwrap())));
        }

        if s.public_key_multibase.is_some() {
            return Ok(Some(KeyFormat::Multibase(s.public_key_multibase.unwrap())));
        }

        if s.public_key_jwk.is_some() {
            return Ok(Some(KeyFormat::Jwk(s.public_key_jwk.unwrap())));
        }

        Ok(None)
    }

    /// Deserializes the private key format from a JSON map with the appropriate key format field.
    pub fn deserialize_private_key_format<'de, D>(deserializer: D) -> Result<Option<KeyFormat>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct PrivateKeyFormat {
            private_key_base58: Option<String>,
            private_key_multibase: Option<String>,
            private_key_jwk: Option<Jwk>,
        }

        let s: PrivateKeyFormat = PrivateKeyFormat::deserialize(deserializer)?;

        if s.private_key_base58.is_some() {
            return Ok(Some(KeyFormat::Base58(s.private_key_base58.unwrap())));
        }

        if s.private_key_multibase.is_some() {
            return Ok(Some(KeyFormat::Multibase(s.private_key_multibase.unwrap())));
        }

        if s.private_key_jwk.is_some() {
            return Ok(Some(KeyFormat::Jwk(s.private_key_jwk.unwrap())));
        }

        Ok(None)
    }
}

/// Represents the proofs in a DID Document.
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Proofs {
    SingleProof(Box<Proof>),
    SetOfProofs(Vec<Proof>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_jwk::Key;
    use multibase::Base::Base64Url;

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
    // The context entry in this case is a single string value.
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

    // DID document with a controller property
    // The context entry in this case is an object and a string
    // The json document is non conform, has it is carying a trailing comma. Must be cleaned before processing.
    // See https://www.w3.org/TR/did-core/#example-did-document-with-a-controller-property
    #[test]
    fn test_canonicalize_didcore_example_12() {
        read_write_did(
            "test_resources/didcore_example_12_context_object.json",
            "test_resources/didcore_example_12_context_object_canonicalized.json",
        )
        .unwrap();
    }

    // DID document with a controller property
    // The context entry in this case just an object
    // The json document is non conform, has it is carying a trailing comma. Must be cleaned before processing.
    // See https://www.w3.org/TR/did-core/#example-did-document-with-a-controller-property
    #[test]
    fn test_canonicalize_didcore_example_12_1() {
        read_write_did(
            "test_resources/didcore_example_12_1_context_object.json",
            "test_resources/didcore_example_12_1_context_object_canonicalized.json",
        )
        .unwrap();
    }

    // DID document with a controller property
    // The context entry in this case is a multiple string values.
    // The json document is non conform, has it is carying a trailing comma. Must be cleaned before processing.
    // See https://www.w3.org/TR/did-core/#example-did-document-with-a-controller-property
    #[test]
    fn test_canonicalize_didcore_example_12_2() {
        read_write_did(
            "test_resources/didcore_example_12_2_context_multiple_strings.json",
            "test_resources/didcore_example_12_2_context_multiple_strings_canonicalized.json",
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

    // Tests that verification methods are properly deserialized
    // with regard to public/private keys
    #[test]
    fn test_deserialize_verification_method() {
        let vm: VerificationMethod = serde_json::from_str(
            r#"{
                "id": "did:web:localhost#keys-1",
                "type": "JsonWebKey2020",
                "controller": "did:web:localhost",
                "publicKeyMultibase": "zH3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV",
                "privateKeyBase58": "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
            }"#,
        )
        .unwrap();

        assert!(vm.public_key.is_some());
        assert!(matches!(vm.public_key.unwrap(), KeyFormat::Multibase(_)));

        assert!(vm.private_key.is_some());
        assert!(matches!(vm.private_key.unwrap(), KeyFormat::Base58(_)));

        let vm: VerificationMethod = serde_json::from_str(
            r#"{
                "id": "did:web:localhost#keys-1",
                "type": "JsonWebKey2020",
                "controller": "did:web:localhost",
                "publicKeyJwk": {
                    "kty": "OKP",
                    "crv": "X25519",
                    "x": "psQvZbwHAW4z2wrTKGbl4mFyzSIGy_Cw7ov-ep0TWAM"
                },
                "privateKeyJwk": {
                    "kty": "OKP",
                    "crv": "X25519",
                    "x": "psQvZbwHAW4z2wrTKGbl4mFyzSIGy_Cw7ov-ep0TWAM",
                    "d": "bBuzzQqaC29xi78lZUWLcByvm7vKgTJqsZ8m7T7KSOw"
                }
            }"#,
        )
        .unwrap();

        assert!(vm.public_key.is_some());
        assert!(vm.private_key.is_some());

        match vm.public_key.unwrap() {
            KeyFormat::Jwk(jwk) => {
                let public_key: Vec<u8> = match jwk.key {
                    Key::Okp(okp) => okp.x.to_vec(),
                    _ => panic!("Unexpected key type"),
                };
                assert_eq!(public_key, Base64Url.decode("psQvZbwHAW4z2wrTKGbl4mFyzSIGy_Cw7ov-ep0TWAM").unwrap());
            }
            _ => panic!("Deserialized into wrong KeyFormat"),
        }

        match vm.private_key.unwrap() {
            KeyFormat::Jwk(jwk) => {
                let private_key = match jwk.key {
                    Key::Okp(okp) => {
                        if let Some(secret) = okp.d {
                            secret.to_vec()
                        } else {
                            panic!("Private key is missing");
                        }
                    }
                    _ => panic!("Unexpected key type"),
                };

                assert_eq!(private_key, Base64Url.decode("bBuzzQqaC29xi78lZUWLcByvm7vKgTJqsZ8m7T7KSOw").unwrap());
            }
            _ => panic!("Deserialized into wrong KeyFormat"),
        }
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
