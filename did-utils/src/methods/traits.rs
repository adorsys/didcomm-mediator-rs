use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{didcore::Document as DIDDocument, ldmodel::Context, methods::errors::DIDResolutionError};

/////////////////////////////////////////////////////////////////////////////////////
///  DID METHOD  -----------------------------------------------------------------///
/////////////////////////////////////////////////////////////////////////////////////

/// Abstract contract for DID methods.
///
/// Initially thought to encompass the signatures of different operations
/// that a DID method is optionally expected to support, it eventually
/// turned out DID methods might be too specific in their underlying modus
/// operandus that such signatures would be counterproductive.
///
/// TODO! Enrich this common interface.
pub trait DIDMethod: DIDResolver {
    /// Returns the DIDMethod's registered name, prefixed with `did:`,
    /// e.g. did:key, did:web, etc.
    fn name() -> String;

    /// Extracts the supertrait resolver object.
    fn resolver(&self) -> &dyn DIDResolver
    where
        Self: Sized,
    {
        self
    }
}

/////////////////////////////////////////////////////////////////////////////////////
///  DID RESOLUTION  -------------------------------------------------------------///
/////////////////////////////////////////////////////////////////////////////////////

/// Abstract contract for DID resolution.
///
/// See https://w3c-ccg.github.io/did-resolution.
#[async_trait]
pub trait DIDResolver {
    /// Resolves a DID address into its corresponding DID document.
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput;

    /// Dereferences a DID URL into its corresponding resource.
    async fn dereference(&self, did_url: &str, _options: &DereferencingOptions) -> DereferencingOutput {
        let context = Context::SingleString(String::from("https://w3id.org/did-resolution/v1"));

        let res = super::utils::parse_did_url(did_url);
        if res.is_err() {
            return DereferencingOutput {
                context,
                content: None,
                dereferencing_metadata: Some(DIDResolutionMetadata {
                    error: Some(DIDResolutionError::InvalidDidUrl),
                    content_type: None,
                    additional_properties: None,
                }),
                content_metadata: None,
                additional_properties: None,
            };
        }

        let (did, query, fragment) = res.unwrap();

        let resolution_output = self.resolve(&did, _options).await;
        if resolution_output.did_resolution_metadata.is_some() {
            let error = resolution_output.did_resolution_metadata.as_ref().unwrap().error.clone();
            if let Some(err) = error {
                return DereferencingOutput {
                    context,
                    content: None,
                    dereferencing_metadata: Some(DereferencingMetadata {
                        error: if err == DIDResolutionError::InvalidDid {
                            Some(DIDResolutionError::InvalidDidUrl)
                        } else {
                            Some(err)
                        },
                        content_type: None,
                        additional_properties: None,
                    }),
                    content_metadata: None,
                    additional_properties: None,
                };
            }
        };

        let diddoc = match &resolution_output.did_document {
            Some(doc) => doc,
            None => unreachable!("DID document must be present if no DIDResolutionError"),
        };

        match dereference_did_document(diddoc, &query, &fragment) {
            Ok(content) => DereferencingOutput {
                context,
                content: Some(content.clone()),
                dereferencing_metadata: Some(DereferencingMetadata {
                    error: None,
                    content_type: match content {
                        Content::DIDDocument(_) => {
                            if resolution_output.did_resolution_metadata.is_some() {
                                resolution_output.did_resolution_metadata.unwrap().content_type
                            } else {
                                None
                            }
                        }
                        _ => Some(MediaType::Json.to_string()),
                    },
                    additional_properties: None,
                }),
                content_metadata: None,
                additional_properties: None,
            },
            Err(err) => DereferencingOutput {
                context,
                content: None,
                dereferencing_metadata: Some(DereferencingMetadata {
                    error: Some(err),
                    content_type: None,
                    additional_properties: None,
                }),
                content_metadata: None,
                additional_properties: None,
            },
        }
    }
}

/// DID Resolution Options.
///
/// Formerly known as "DID resolution input metadata", they provide
/// additional configuration for the DID resolution process.
///
/// See https://www.w3.org/TR/did-core/#did-resolution-options
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DIDResolutionOptions {
    // See https://www.w3.org/TR/did-spec-registries/#accept
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<MediaType>,
    // See https://w3c-ccg.github.io/did-resolution/#caching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_cache: Option<bool>,
    // Dynamic properties
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// DID Resolution Output.
///
/// See https://w3c-ccg.github.io/did-resolution/#did-resolution-result
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
/// See https://www.w3.org/TR/did-core/#did-resolution-metadata
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
/// See https://www.w3.org/TR/did-core/#did-document-metadata
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
/// See https://www.w3.org/TR/did-core/#did-url-dereferencing-options
pub type DereferencingOptions = DIDResolutionOptions;

/// DID URL Dereferencing Metadata.
///
/// See https://www.w3.org/TR/did-core/#did-url-dereferencing-metadata
pub type DereferencingMetadata = DIDResolutionMetadata;

/// Content Metadata.
///
/// See https://www.w3.org/TR/did-core/#metadata-structure
pub type ContentMetadata = DIDDocumentMetadata;

/// Dereferencing Output.
///
/// See https://w3c-ccg.github.io/did-resolution/#did-resolution-result
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
    // See https://www.w3.org/TR/did-core/#dfn-didresolutionmetadata
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

/// Serves derefencing query given a DID document
fn dereference_did_document(diddoc: &DIDDocument, query: &HashMap<String, String>, fragment: &Option<String>) -> Result<Content, DIDResolutionError> {
    // Primary resource
    if query.contains_key("service") {
        let mut found = vec![];

        let service = query.get("service").unwrap();
        for entry in diddoc.service.clone().unwrap_or(vec![]) {
            if entry.id.ends_with(&format!("#{service}")) {
                found.push(entry.service_endpoint);
            }
        }

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
            "{found}{}{}",
            match relative_ref {
                Some(val) => val,
                None => "",
            },
            match fragment {
                Some(frag) => format!("#{frag}"),
                None => String::new(),
            }
        )));
    } else if !query.is_empty() {
        // Resort to returning whole DID document as other query parameters
        // are not supported by this default dereferencing implementation.
        return Ok(Content::DIDDocument(diddoc.clone()));
    }

    // Secondary resource without primary resource
    if fragment.is_some() {
        let mut found = vec![];
        let needle = format!("{}#{}", diddoc.id, fragment.as_ref().unwrap());

        let haystack = [
            json!(diddoc.authentication.clone().unwrap_or(vec![])),
            json!(diddoc.assertion_method.clone().unwrap_or(vec![])),
            json!(diddoc.key_agreement.clone().unwrap_or(vec![])),
            json!(diddoc.verification_method.clone().unwrap_or(vec![])),
            json!(diddoc.service.clone().unwrap_or(vec![])),
        ];

        for entry in haystack {
            for vm in entry.as_array().unwrap() {
                let id = vm.get("id");
                if id.is_some() && id.unwrap().as_str().unwrap() == needle {
                    found.push(vm.clone());
                }
            }
        }

        if found.is_empty() {
            return Err(DIDResolutionError::NotFound);
        }

        if found.len() > 1 {
            return Err(DIDResolutionError::NotAllowedLocalDuplicateKey);
        }

        return Ok(Content::Data(found.into_iter().next().unwrap()));
    }

    // Resort to returning whole DID document
    Ok(Content::DIDDocument(diddoc.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::utils::parse_did_url;

    #[async_std::test]
    async fn test_dereferencing_did_url() {
        let diddoc: DIDDocument = serde_json::from_str(
            r#"{
                "@context": "https://www.w3.org/ns/did/v1",
                "id": "did:example:123456789abcdefghi",
                "verificationMethod": [{
                    "id": "did:example:123456789abcdefghi#keys-1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "did:example:123456789abcdefghi",
                    "publicKeyBase58": "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
                }],
                "service": [
                    {
                        "id": "did:example:123456789abcdefghi#agent",
                        "type": "AgentService",
                        "serviceEndpoint": "https://agent.example.com/8377464"
                    },
                    {
                        "id": "did:example:123456789abcdefghi#agent",
                        "type": "DuplicateAgentService",
                        "serviceEndpoint": "https://agent.example.com/8377465"
                    },
                    {
                        "id": "did:example:123456789abcdefghi#client",
                        "type": "ClientService",
                        "serviceEndpoint": "https://client.example.com/8377467#real"
                    },
                    {
                        "id": "did:example:123456789abcdefghi#messages",
                        "type": "MessagingService",
                        "serviceEndpoint": "https://example.com/messages/8377464"
                    }
                ]
            }"#,
        )
        .unwrap();

        let happy_cases = [
            (
                "did:example:123456789abcdefghi#keys-1",
                r#"{
                    "id": "did:example:123456789abcdefghi#keys-1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "did:example:123456789abcdefghi",
                    "publicKeyBase58": "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
                }"#,
            ),
            (
                "did:example:123456789abcdefghi#client",
                r#"{
                    "id": "did:example:123456789abcdefghi#client",
                    "type": "ClientService",
                    "serviceEndpoint": "https://client.example.com/8377467#real"
                }"#,
            ),
            (
                "did:example:123456789abcdefghi?service=messages&relativeRef=%2Fsome%2Fpath%3Fquery#frag",
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
            ("did:example:123456789abcdefghi#unknown", DIDResolutionError::NotFound),
            ("did:example:123456789abcdefghi?service=unknown", DIDResolutionError::NotFound),
            ("did:example:123456789abcdefghi#agent", DIDResolutionError::NotAllowedLocalDuplicateKey),
            (
                "did:example:123456789abcdefghi?service=agent",
                DIDResolutionError::NotAllowedLocalDuplicateKey,
            ),
            (
                "did:example:123456789abcdefghi?service=client&relativeRef=something",
                DIDResolutionError::InternalError,
            ),
            (
                "did:example:123456789abcdefghi?service=client#something",
                DIDResolutionError::InternalError,
            ),
        ];

        for (did_url, expected) in corner_cases {
            let (_, query, fragment) = parse_did_url(did_url).unwrap();
            let output = dereference_did_document(&diddoc, &query, &fragment).unwrap_err();

            assert_eq!(output, expected);
        }
    }
}
