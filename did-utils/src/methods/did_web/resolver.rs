//! A module for resolving DID Web documents using HTTP and HTTPS schemes.
//!
//! This module provides functionnalities for resolving DID Web documents by fetching
//! them over HTTP or HTTPS. The resolver follows the [W3C DID Resolution specification].
//! 
//! [W3C DID Resolution specification]: https://w3c.github.io/did-resolution/

use async_trait::async_trait;
use hyper::{
    client::{ connect::Connect, HttpConnector },
    http::uri::{ self, Scheme },
    Body,
    Client,
    Uri,
};
use hyper_tls::HttpsConnector;

use crate::methods::{
    errors::DidWebError,
    traits::{
        DIDResolutionMetadata,
        DIDResolutionOptions,
        DIDResolver,
        MediaType,
        ResolutionOutput,
    },
};

use crate::ldmodel::Context;

use crate::didcore::Document as DIDDocument;

/// A struct for resolving DID Web documents.
pub struct DidWebResolver<C> where C: Connect + Send + Sync + Clone + 'static {
    client: Client<C>,
    scheme: Scheme,
}

impl DidWebResolver<HttpConnector> {
    
    /// Creates a new `DidWebResolver` with the default HTTP scheme.
    pub fn http() -> DidWebResolver<HttpConnector> {
        DidWebResolver {
            client: Client::builder().build::<_, Body>(HttpConnector::new()),
            scheme: Scheme::HTTP,
        }
    }
}

impl DidWebResolver<HttpsConnector<HttpConnector>> {

    /// Creates a new `DidWebResolver` with the HTTPS scheme.
    pub fn https() -> DidWebResolver<HttpsConnector<HttpConnector>> {
        DidWebResolver {
            client: Client::builder().build::<_, Body>(HttpsConnector::new()),
            scheme: Scheme::HTTPS,
        }
    }
}

impl<C> DidWebResolver<C> where C: Connect + Send + Sync + Clone + 'static {

    /// Fetches a DID document from the given URL.
    ///
    /// This method performs an HTTP GET request to the provided URL
    /// and attempts to returns the response body as a string.
    /// 
    /// # Arguments
    ///
    /// * `url` - The URL to fetch the DID document from.
    ///
    /// # Returns
    ///
    /// A `Result` containing the DID document as a string or a `DidWebError`.
    async fn fetch_did_document(&self, url: Uri) -> Result<String, DidWebError> {
        let res = self.client.get(url).await?;

        if !res.status().is_success() {
            return Err(DidWebError::NonSuccessResponse(res.status()));
        }

        let body = hyper::body::to_bytes(res.into_body()).await?;

        String::from_utf8(body.to_vec()).map_err(|err| err.into())
    }
}

impl<C> DidWebResolver<C> where C: Connect + Send + Sync + Clone + 'static {

    /// Fetches and parses a DID document for the given DID.
    ///
    /// This method first parses the DID Web URL format from the given DID and then constructs
    /// an URI based on the scheme, domain name, and path. It then fetches the DID document and
    /// parses the response body.
    /// 
    /// # Arguments
    ///
    /// * `did` - The DID to resolve.
    ///
    /// # Returns
    ///
    /// A `Result` containing the resolved `DIDDocument` or a `DidWebError`.
    async fn resolver_fetcher(&self, did: &str) -> Result<DIDDocument, DidWebError> {
        let (path, domain_name) = match parse_did_web_url(did) {
            Ok((path, domain_name)) => (path, domain_name),
            Err(err) => {
                return Err(DidWebError::RepresentationNotSupported(err.to_string()));
            }
        };

        let url: Uri = match
            uri::Builder
                ::new()
                .scheme(self.scheme.clone())
                .authority(domain_name)
                .path_and_query(path)
                .build()
        {
            Ok(url) => url,
            Err(err) => {
                return Err(DidWebError::RepresentationNotSupported(err.to_string()));
            }
        };

        let json_string = match self.fetch_did_document(url).await {
            Ok(json) => json,
            Err(err) => {
                return Err(err);
            }
        };

        let did_document: DIDDocument = match serde_json::from_str(&json_string) {
            Ok(document) => document,
            Err(err) => {
                return Err(DidWebError::RepresentationNotSupported(err.to_string()));
            }
        };

        Ok(did_document)
    }
}

/// Parses a DID Web URL and returns the path and domain name.
///
/// # Arguments
///
/// * `did` - The DID to parse.
///
/// # Returns
///
/// A `Result` containing the path and domain name or a `DidWebError`.
pub fn parse_did_web_url(did: &str) -> Result<(String, String), DidWebError> {
    let mut parts = did.split(':').peekable();
    let domain_name = match (parts.next(), parts.next(), parts.next()) {
        (Some("did"), Some("web"), Some(domain_name)) => domain_name.replacen("%3A", ":", 1),
        _ => {
            return Err(DidWebError::InvalidDid("Invalid DID".to_string()));
        }
    };

    let mut path = match parts.peek() {
        Some(_) => parts.collect::<Vec<&str>>().join("/"),
        None => ".well-known".to_string(),
    };

    path = format!("/{}/did.json", path);

    Ok((path, domain_name))
}

#[async_trait]
impl<C> DIDResolver for DidWebResolver<C> where C: Connect + Send + Sync + Clone + 'static {

    /// Resolves a DID to a DID document.
    ///
    /// # Arguments
    ///
    /// * `did` - The DID to resolve.
    /// * `_options` - The options for DID resolution.
    ///
    /// # Returns
    ///
    /// A `ResolutionOutput` containing the resolved DID document and metadata.
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput {
        let context = Context::SingleString(String::from("https://www.w3.org/ns/did/v1"));

        match self.resolver_fetcher(did).await {
            Ok(diddoc) =>
                ResolutionOutput {
                    context,
                    did_document: Some(diddoc),
                    did_resolution_metadata: Some(DIDResolutionMetadata {
                        error: None,
                        content_type: Some(MediaType::DidLdJson.to_string()),
                        additional_properties: None,
                    }),
                    did_document_metadata: None,
                    additional_properties: None,
                },
            Err(_err) =>
                ResolutionOutput {
                    context,
                    did_document: None,
                    did_resolution_metadata: None,
                    did_document_metadata: None,
                    additional_properties: None,
                },
        }
    }
}
