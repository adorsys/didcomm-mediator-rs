use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
/// TODO! Add support for dereferencing.
#[async_trait]
pub trait DIDResolver {
    /// Resolves a DID address into its corresponding DID document.
    async fn resolve(&self, did: &str, options: &DIDResolutionOptions) -> ResolutionOutput;
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
    // See https://www.w3.org/TR/did-spec-registries/#versionId-param
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    // See https://www.w3.org/TR/did-spec-registries/#versionTime-param
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_time: Option<String>,
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
    created: Option<DateTime<Utc>>,
    // See https://www.w3.org/TR/did-spec-registries/#updated
    #[serde(skip_serializing_if = "Option::is_none")]
    updated: Option<DateTime<Utc>>,
    // See https://www.w3.org/TR/did-spec-registries/#deactivated
    #[serde(skip_serializing_if = "Option::is_none")]
    deactivated: Option<bool>,
    // See https://www.w3.org/TR/did-spec-registries/#next_update
    #[serde(skip_serializing_if = "Option::is_none")]
    next_update: Option<DateTime<Utc>>,
    // See https://www.w3.org/TR/did-spec-registries/#version_id
    #[serde(skip_serializing_if = "Option::is_none")]
    version_id: Option<String>,
    // See https://www.w3.org/TR/did-spec-registries/#next_version_id
    #[serde(skip_serializing_if = "Option::is_none")]
    next_version_id: Option<String>,
    // See https://www.w3.org/TR/did-spec-registries/#equivalent_id
    #[serde(skip_serializing_if = "Vec::is_empty")]
    equivalent_id: Vec<String>,
    // See https://www.w3.org/TR/did-spec-registries/#canonical_id
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_id: Option<String>,
    // Dynamic properties
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Media type for resolution input and output metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MediaType {
    DidJson,
    DidLdJson,
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MediaType::DidJson => write!(f, "application/did+json"),
            MediaType::DidLdJson => write!(f, "application/did+ld+json"),
        }
    }
}
