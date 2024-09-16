use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{didcore::Document as DIDDocument, ldmodel::Context, methods::errors::DIDResolutionError};

/// DID Resolution Options.
///
/// Formerly known as "DID resolution input metadata", they provide
/// additional configuration for the DID resolution process.
///
/// See `<https://www.w3.org/TR/did-core/#did-resolution-options>`
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DIDResolutionOptions {
    // See https://www.w3.org/TR/did-spec-registries/#accept
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<MediaType>,
    // See https://w3c.github.io/did-resolution/#caching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_cache: Option<bool>,
    // Dynamic properties
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// DID Resolution Output.
///
/// See `<https://www.w3.org/TR/did-core/#did-resolution>`
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionOutput {
    // The @context property defines the vocabulary used in the JSON-LD document.
    // It provides a way to map the keys in the JSON structure to specific terms,
    // properties, and classes from external vocabularies.
    #[serde(rename = "@context")]
    pub context: Context,
    // See https://www.w3.org/TR/did-core/#dfn-diddocument
    pub did_document: Option<DIDDocument>,
    // See https://www.w3.org/TR/did-core/#dfn-didresolutionmetadata
    pub did_resolution_metadata: Option<DIDResolutionMetadata>,
    // See https://www.w3.org/TR/did-core/#dfn-diddocumentmetadata
    pub did_document_metadata: Option<DIDDocumentMetadata>,
    // Dynamic properties
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// DID Resolution Metadata.
///
/// See `<https://www.w3.org/TR/did-core/#did-resolution-metadata>`
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DIDResolutionMetadata {
    // See https://www.w3.org/TR/did-spec-registries/#error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<DIDResolutionError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // See https://www.w3.org/TR/did-spec-registries/#contenttype
    pub content_type: Option<String>,
    // Dynamic properties
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// DID Document Metadata.
///
/// See `<https://www.w3.org/TR/did-core/#did-document-metadata>`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DIDDocumentMetadata {
    // See https://www.w3.org/TR/did-spec-registries/#created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    // See https://www.w3.org/TR/did-spec-registries/#updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
    // See https://www.w3.org/TR/did-spec-registries/#deactivated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deactivated: Option<bool>,
    // See https://www.w3.org/TR/did-spec-registries/#next_update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_update: Option<DateTime<Utc>>,
    // See https://www.w3.org/TR/did-spec-registries/#version_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    // See https://www.w3.org/TR/did-spec-registries/#next_version_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_version_id: Option<String>,
    // See https://www.w3.org/TR/did-spec-registries/#equivalent_id
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub equivalent_id: Vec<String>,
    // See https://www.w3.org/TR/did-spec-registries/#canonical_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_id: Option<String>,
    // Dynamic properties
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// DID URL Dereferencing Options.
///
/// See `<https://www.w3.org/TR/did-core/#did-url-dereferencing-options>`
pub type DereferencingOptions = DIDResolutionOptions;

/// DID URL Dereferencing Metadata.
///
/// See `<https://www.w3.org/TR/did-core/#did-url-dereferencing-metadata>`
pub type DereferencingMetadata = DIDResolutionMetadata;

/// Content Metadata.
///
/// See `<https://www.w3.org/TR/did-core/#metadata-structure>`
pub type ContentMetadata = DIDDocumentMetadata;

/// Dereferencing Output.
///
/// See `<https://www.w3.org/TR/did-core/#did-url-dereferencing>`
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DereferencingOutput {
    // The @context property defines the vocabulary used in the JSON-LD document.
    // It provides a way to map the keys in the JSON structure to specific terms,
    // properties, and classes from external vocabularies.
    #[serde(rename = "@context")]
    pub context: Context,
    // See https://www.w3.org/TR/did-core/#dfn-diddocument
    pub content: Option<Content>,
    // See https://www.w3.org/TR/did-core/#did-url-dereferencing-metadata
    pub dereferencing_metadata: Option<DereferencingMetadata>,
    // See https://www.w3.org/TR/did-core/#dfn-diddocumentmetadata
    pub content_metadata: Option<ContentMetadata>,
    // Dynamic properties
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// A resource returned by DID URL dereferencing
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Content {
    /// DID Document
    DIDDocument(DIDDocument),
    /// URL
    URL(String),
    /// Other (e.g. verification method map)
    Data(Value),
}

/// Media type for resolution input and output metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MediaType {
    Json,
    DidJson,
    DidLdJson,
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MediaType::Json => write!(f, "application/json"),
            MediaType::DidJson => write!(f, "application/did+json"),
            MediaType::DidLdJson => write!(f, "application/did+ld+json"),
        }
    }
}

/// Serves derefencing query given a DID document.
pub(super) fn dereference_did_document(
    diddoc: &DIDDocument,
    query: &HashMap<String, String>,
    fragment: &Option<String>,
) -> Result<Content, DIDResolutionError> {
    // Primary resource
    if let Some(service) = query.get("service") {
        let entries = diddoc.service.clone().unwrap_or_default();
        let found: Vec<_> = entries
            .iter()
            .filter(|entry| entry.id.ends_with(&format!("#{}", service)))
            .map(|entry| entry.service_endpoint.clone())
            .collect();

        if found.is_empty() {
            return Err(DIDResolutionError::NotFound);
        }

        if found.len() > 1 {
            return Err(DIDResolutionError::NotAllowedLocalDuplicateKey);
        }

        let found = &found[0];
        let relative_ref = query.get("relativeRef");
        if (fragment.is_some() || relative_ref.is_some()) && found.contains('#') {
            return Err(DIDResolutionError::InternalError);
        }

        return Ok(Content::URL(format!(
            "{}{}{}",
            found,
            relative_ref.unwrap_or(&String::new()),
            fragment.as_ref().map_or(String::new(), |frag| format!("#{frag}"))
        )));
    } else if !query.is_empty() {
        // Resort to returning whole DID document as other query parameters
        // are not supported by this default dereferencing implementation.
        return Ok(Content::DIDDocument(diddoc.clone()));
    }

    // Secondary resource without primary resource
    if let Some(fragment) = fragment {
        let needle = format!("{}#{}", diddoc.id, fragment);

        let haystack = [
            json!(diddoc.authentication.as_ref().unwrap_or(&vec![])),
            json!(diddoc.assertion_method.as_ref().unwrap_or(&vec![])),
            json!(diddoc.key_agreement.as_ref().unwrap_or(&vec![])),
            json!(diddoc.verification_method.as_ref().unwrap_or(&vec![])),
            json!(diddoc.service.as_ref().unwrap_or(&vec![])),
        ];

        let flat_haystack = haystack.iter().flat_map(|x| x.as_array().unwrap());
        let found: Vec<_> = flat_haystack.filter(|vm| vm.get("id") == Some(&json!(&needle))).collect();

        if found.is_empty() {
            return Err(DIDResolutionError::NotFound);
        }

        if found.len() > 1 {
            return Err(DIDResolutionError::NotAllowedLocalDuplicateKey);
        }

        return Ok(Content::Data(found[0].clone()));
    }

    // Resort to returning whole DID document
    Ok(Content::DIDDocument(diddoc.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::utils::parse_did_url;

    #[test]
    fn test_dereference_did_document_primary_resource() {
        let diddoc = create_sample_did_document();

        let query = [("service".to_string(), "service-1".to_string())].into_iter().collect();
        let fragment: Option<String> = None;

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert_eq!(result.unwrap(), Content::URL("https://example.com/service-1".to_string()));
    }

    #[test]
    fn test_dereference_did_document_primary_resource_not_found() {
        let diddoc = create_sample_did_document();

        let query = [("service".to_string(), "non-existent".to_string())].into_iter().collect();
        let fragment: Option<String> = None;

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert!(matches!(result, Err(DIDResolutionError::NotFound)));
    }

    #[test]
    fn test_dereference_did_document_primary_resource_duplicate_key() {
        let diddoc = create_sample_did_document();

        let query = [("service".to_string(), "service-dup".to_string())].into_iter().collect();
        let fragment: Option<String> = None;

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert!(matches!(result, Err(DIDResolutionError::NotAllowedLocalDuplicateKey)));
    }

    #[test]
    fn test_dereference_did_document_primary_resource_with_relative_ref() {
        let diddoc = create_sample_did_document();

        let query = [
            ("service".to_string(), "service-1".to_string()),
            ("relativeRef".to_string(), "/some/path?k=v".to_string()),
        ]
        .into_iter()
        .collect();
        let fragment: Option<String> = None;

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert_eq!(result.unwrap(), Content::URL("https://example.com/service-1/some/path?k=v".to_string()));
    }

    #[test]
    fn test_dereference_did_document_primary_resource_with_fragment() {
        let diddoc = create_sample_did_document();

        let query = [("service".to_string(), "service-1".to_string())].into_iter().collect();
        let fragment: Option<String> = Some("frag".to_string());

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert_eq!(result.unwrap(), Content::URL("https://example.com/service-1#frag".to_string()));
    }

    #[test]
    fn test_dereference_did_document_primary_resource_with_relative_ref_and_fragment() {
        let diddoc = create_sample_did_document();

        let query = [
            ("service".to_string(), "service-1".to_string()),
            ("relativeRef".to_string(), "/some/path?k=v".to_string()),
        ]
        .into_iter()
        .collect();
        let fragment: Option<String> = Some("frag".to_string());

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert_eq!(
            result.unwrap(),
            Content::URL("https://example.com/service-1/some/path?k=v#frag".to_string())
        );
    }

    #[test]
    fn test_dereference_did_document_primary_resource_with_unappendable_fragment() {
        let diddoc = create_sample_did_document();

        let query = [("service".to_string(), "client".to_string())].into_iter().collect();
        let fragment: Option<String> = Some("fragment-1".to_string());

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert!(matches!(result, Err(DIDResolutionError::InternalError)));
    }

    #[test]
    fn test_dereference_did_document_primary_resource_with_unappendable_relative_ref() {
        let diddoc = create_sample_did_document();

        let query = [
            ("service".to_string(), "client".to_string()),
            ("relativeRef".to_string(), "?k=v".to_string()),
        ]
        .into_iter()
        .collect();
        let fragment: Option<String> = None;

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert!(matches!(result, Err(DIDResolutionError::InternalError)));
    }

    #[test]
    fn test_dereference_did_document_primary_resource_with_no_service_parameter() {
        let diddoc = create_sample_did_document();

        let query = [("key".to_string(), "value".to_string())].into_iter().collect();
        let fragment: Option<String> = None;

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert!(matches!(result, Ok(Content::DIDDocument(_))));
        assert_eq!(
            json_canon::to_string(&result.unwrap()).unwrap(), //
            json_canon::to_string(&diddoc).unwrap(),          //
        );
    }

    #[test]
    fn test_dereference_did_document_primary_resource_with_neither_query_nor_fragment() {
        let diddoc = create_sample_did_document();

        let query = HashMap::new();
        let fragment: Option<String> = None;

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert!(matches!(result, Ok(Content::DIDDocument(_))));
        assert_eq!(
            json_canon::to_string(&result.unwrap()).unwrap(), //
            json_canon::to_string(&diddoc).unwrap(),          //
        );
    }

    #[test]
    fn test_dereference_did_document_secondary_resource() {
        let diddoc = create_sample_did_document();

        let cases = [
            (
                "auth-1",
                r#"{"id": "did:example:123#auth-1", "type": "Ed25519VerificationKey2020", "controller": "did:example:123"}"#,
            ),
            (
                "keys-1",
                r#"{"id": "did:example:123#keys-1", "type": "Ed25519VerificationKey2018", "controller": "did:example:123"}"#,
            ),
        ];

        for (fragment, expected) in cases {
            let fragment: Option<String> = Some(fragment.to_string());
            let expected: Value = serde_json::from_str(expected).unwrap();

            let result = dereference_did_document(&diddoc, &HashMap::new(), &fragment);
            assert!(matches!(result, Ok(Content::Data(_))));
            assert_eq!(
                json_canon::to_string(&result.unwrap()).unwrap(), //
                json_canon::to_string(&expected).unwrap(),        //
            );
        }
    }

    #[test]
    fn test_dereference_did_document_secondary_resource_not_found() {
        let diddoc = create_sample_did_document();

        let query = HashMap::new();
        let fragment: Option<String> = Some("non-existent".to_string());

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert!(matches!(result, Err(DIDResolutionError::NotFound)));
    }

    #[test]
    fn test_dereference_did_document_secondary_resource_duplicate_key() {
        let diddoc = create_sample_did_document();

        let query = HashMap::new();
        let fragment: Option<String> = Some("keys-dup".to_string());

        let result = dereference_did_document(&diddoc, &query, &fragment);
        assert!(matches!(result, Err(DIDResolutionError::NotAllowedLocalDuplicateKey)));
    }

    #[test]
    fn test_dereferencing_did_url() {
        let diddoc = create_sample_did_document();

        let happy_cases = [
            (
                "did:example:123#keys-1",
                r#"{
                    "id": "did:example:123#keys-1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "did:example:123"
                }"#,
            ),
            (
                "did:example:123#client",
                r#"{
                    "id": "did:example:123#client",
                    "type": "ClientService",
                    "serviceEndpoint": "https://client.example.com/8377467#real"
                }"#,
            ),
            (
                "did:example:123?service=messages&relativeRef=%2Fsome%2Fpath%3Fquery#frag",
                r#""https://example.com/messages/8377464/some/path?query#frag""#,
            ),
        ];

        for (did_url, expected) in happy_cases {
            let expected: Value = serde_json::from_str(expected).unwrap();

            let (_, query, fragment) = parse_did_url(did_url).unwrap();
            let output = dereference_did_document(&diddoc, &query, &fragment).unwrap();

            assert_eq!(
                json_canon::to_string(&output).unwrap(),   //
                json_canon::to_string(&expected).unwrap(), //
            );
        }

        let corner_cases = [
            ("did:example:123#unknown", DIDResolutionError::NotFound),
            ("did:example:123?service=unknown", DIDResolutionError::NotFound),
            ("did:example:123#agent", DIDResolutionError::NotAllowedLocalDuplicateKey),
            ("did:example:123?service=agent", DIDResolutionError::NotAllowedLocalDuplicateKey),
            ("did:example:123?service=client&relativeRef=something", DIDResolutionError::InternalError),
            ("did:example:123?service=client#something", DIDResolutionError::InternalError),
        ];

        for (did_url, expected) in corner_cases {
            let (_, query, fragment) = parse_did_url(did_url).unwrap();
            let output = dereference_did_document(&diddoc, &query, &fragment).unwrap_err();

            assert_eq!(output, expected);
        }
    }

    // Helper function to create a sample DID document for testing
    fn create_sample_did_document() -> DIDDocument {
        serde_json::from_str(
            r#"{
                "@context": "https://www.w3.org/ns/did/v1",
                "id": "did:example:123",
                "authentication": [
                    "did:example:123#keys-1",
                    { "id": "did:example:123#auth-1", "type": "Ed25519VerificationKey2020", "controller": "did:example:123"},
                    { "id": "did:example:123#auth-2", "type": "Ed25519VerificationKey2020", "controller": "did:example:123"},
                    { "id": "did:example:123#keys-dup", "type": "Ed25519VerificationKey2020", "controller": "did:example:123"}
                ],
                "assertionMethod": [
                    "did:example:123#keys-1",
                    { "id": "did:example:123#assert-1", "type": "Ed25519VerificationKey2020", "controller": "did:example:123"},
                    { "id": "did:example:123#assert-2", "type": "Ed25519VerificationKey2020", "controller": "did:example:123"}
                ],
                "keyAgreement": [
                    { "id": "did:example:123#key-agree-1", "type": "X25519VerificationKey2020", "controller": "did:example:123"},
                    { "id": "did:example:123#key-agree-2", "type": "X25519VerificationKey2020", "controller": "did:example:123"}
                ],
                "verificationMethod": [
                    { "id": "did:example:123#keys-1", "type": "Ed25519VerificationKey2018", "controller": "did:example:123"},
                    { "id": "did:example:123#keys-dup", "type": "Ed25519VerificationKey2018", "controller": "did:example:123"}
                ],
                "service": [
                    {
                        "id": "did:example:123#service-1",
                        "type": "ExampleService",
                        "serviceEndpoint": "https://example.com/service-1"
                    },
                    {
                        "id": "did:example:123#service-2",
                        "type": "ExampleService",
                        "serviceEndpoint": "https://example.com/service-2"
                    },
                    {
                        "id": "did:example:123#service-dup",
                        "type": "ExampleService",
                        "serviceEndpoint": "https://example.com/service-dup-1"
                    },
                    {
                        "id": "did:example:123#service-dup",
                        "type": "ExampleService",
                        "serviceEndpoint": "https://example.com/service-dup-2"
                    },
                    {
                        "id": "did:example:123#agent",
                        "type": "AgentService",
                        "serviceEndpoint": "https://agent.example.com/8377464"
                    },
                    {
                        "id": "did:example:123#agent",
                        "type": "DuplicateAgentService",
                        "serviceEndpoint": "https://agent.example.com/8377465"
                    },
                    {
                        "id": "did:example:123#client",
                        "type": "ClientService",
                        "serviceEndpoint": "https://client.example.com/8377467#real"
                    },
                    {
                        "id": "did:example:123#messages",
                        "type": "MessagingService",
                        "serviceEndpoint": "https://example.com/messages/8377464"
                    }
                ]
            }"#,
        )
        .unwrap()
    }
}
