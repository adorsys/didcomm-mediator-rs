use async_trait::async_trait;
use hyper::{
    client::{connect::Connect, HttpConnector},
    http::uri::{self, Scheme},
    Body, Client, Uri,
};
use hyper_tls::HttpsConnector;

use crate::methods::{
    errors::DidWebError,
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
    async fn resolver_fetcher(&self, did: &str) -> Result<DIDDocument, DidWebError> {
        let (path, domain_name) = match parse_did_web_url(did) {
            Ok((path, domain_name)) => (path, domain_name),
            Err(err) => {
                return Err(DidWebError::RepresentationNotSupported(err.to_string()));
            }
        };

        let url: Uri = match uri::Builder::new()
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
impl<C> DIDResolver for DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput {
        let context = Context::SingleString(String::from("https://www.w3.org/ns/did/v1"));

        match self.resolver_fetcher(did).await {
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
            Err(_err) => ResolutionOutput {
                context,
                did_document: None,
                did_resolution_metadata: None,
                did_document_metadata: None,
                additional_properties: None,
            },
        }
    }
}
