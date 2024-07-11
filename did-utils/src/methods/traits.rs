//! Trait definitions for DID methods.

use crate::methods::resolution::*;
use async_trait::async_trait;

use crate::{ldmodel::Context, methods::errors::DIDResolutionError};


/// Abstract contract for DID methods.
///
/// Initially thought to encompass the signatures of different operations
/// that a DID method is optionally expected to support, it eventually
/// turned out DID methods might be too specific in their underlying modus
/// operandus that such signatures would be counterproductive.
///
// TODO! Enrich this common interface.
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


/// Abstract contract for DID resolution.
///
/// [See DID Resolution Specification](https://w3c.github.io/did-resolution)
#[async_trait]
pub trait DIDResolver {
    /// Resolves a DID address into its corresponding DID document.
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput;

    /// Dereferences a DID URL into its corresponding resource.
    async fn dereference(&self, did_url: &str, _options: &DereferencingOptions) -> DereferencingOutput {
        let context = Context::SingleString(String::from("https://www.w3.org/ns/did/v1"));

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
