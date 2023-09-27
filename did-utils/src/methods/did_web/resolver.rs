use async_trait::async_trait;
use hyper::{
    client::{connect::Connect, HttpConnector},
    http::uri::{self, Scheme},
    Body, Client, Uri,
};
use hyper_tls::HttpsConnector;

use crate::methods::{
    errors::{DIDResolutionError, DidWebError, GenericError},
    traits::{DIDResolutionMetadata, DIDResolutionOptions, DIDResolver, MediaType, ResolutionOutput},
};

use crate::ldmodel::Context;

use crate::didcore::Document as DIDDocument;

pub struct DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    client: Client<C>,
    scheme: Scheme,
}

impl DidWebResolver<HttpConnector> {
    pub fn http() -> DidWebResolver<HttpConnector> {
        DidWebResolver {
            client: Client::builder().build::<_, Body>(HttpConnector::new()),
            scheme: Scheme::HTTP,
        }
    }
}

impl DidWebResolver<HttpsConnector<HttpConnector>> {
    pub fn https() -> DidWebResolver<HttpsConnector<HttpConnector>> {
        DidWebResolver {
            client: Client::builder().build::<_, Body>(HttpsConnector::new()),
            scheme: Scheme::HTTPS,
        }
    }
}

impl<C> DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    async fn fetch_did_document(&self, url: Uri) -> Result<String, DidWebError> {
        let res = self.client.get(url).await?;

        if !res.status().is_success() {
            return Err(DidWebError::NonSuccessResponse(res.status()));
        }

        let body = hyper::body::to_bytes(res.into_body()).await?;

        String::from_utf8(body.to_vec()).map_err(|err| err.into())
    }
}

impl<C> DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    async fn resolver_fetcher(&self, domain: String, path_and_query: String) -> Result<DIDDocument, GenericError> {
        let url: Uri = uri::Builder::new()
            .scheme("Test")
            .authority(domain)
            .path_and_query(path_and_query)
            .build()?;

        let did_document: DIDDocument = serde_json::from_str(&self.fetch_did_document(url).await?)?;

        Ok(did_document)
    }
}

#[async_trait]
impl<C> DIDResolver for DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput {
        // Split the DID string by ':' and collect the parts into a vector.
        let did_parts: Vec<&str> = did.split(':').collect::<Vec<&str>>();

        let domain: String = did_parts[0].replace("%3A", ":");

        let path_parts = &did_parts[1..];
        let path_and_query = if path_parts.is_empty() {
            "/.well-known/did.json".to_string()
        } else {
            let path = path_parts.join("/");
            format!("/{}/did.json", path)
        };

        let context = Context::SingleString(String::from("https://w3id.org/did-resolution/v1"));

        match self.resolver_fetcher(domain, path_and_query).await {
            Ok(diddoc) => ResolutionOutput {
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
            Err(err) => ResolutionOutput {
                context,
                did_document: None,
                did_resolution_metadata: None,
                did_document_metadata: None,
                additional_properties: None,
            },
        }
    }
}
