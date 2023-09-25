use async_trait::async_trait;
use hyper::{
    client::{connect::Connect, HttpConnector},
    http::uri::Scheme,
    Body, Client, Uri,
};
use hyper_tls::HttpsConnector;

use crate::methods::{
    errors::DidWebError,
    traits::{DIDResolutionOptions, DIDResolver, ResolutionOutput},
};

use crate::ldmodel::Context;

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

#[async_trait]
impl<C> DIDResolver for DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput {
        let context = Context::SingleString(String::from("https://w3id.org/did-resolution/v1"));
        ResolutionOutput {
            context: context,
            did_document: None,
            did_resolution_metadata: None,
            did_document_metadata: None,
            additional_properties: None,
        }
    }
}
