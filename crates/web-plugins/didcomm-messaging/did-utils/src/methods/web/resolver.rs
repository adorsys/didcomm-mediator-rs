use crate::methods::resolution::{DIDResolutionMetadata, DIDResolutionOptions, MediaType, ResolutionOutput};
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::Bytes,
    http::uri::{self, Scheme},
    Uri,
};
use hyper_tls::HttpsConnector;
use hyper_util::{
    client::legacy::{
        connect::{Connect, HttpConnector},
        Client,
    },
    rt::TokioExecutor,
};

use crate::ldmodel::Context;
use crate::methods::{errors::DidWebError, traits::DIDResolver};

use crate::didcore::Document as DIDDocument;

/// A struct for resolving DID Web documents.
pub struct DidWeb<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    client: Client<C, Full<Bytes>>,
}

impl DidWeb<HttpConnector> {
    // Creates a new `DidWeb` resolver with HTTP scheme, for testing only.
    #[cfg(test)]
    pub fn http() -> DidWeb<HttpConnector> {
        DidWeb {
            client: Client::builder(TokioExecutor::new()).build_http(),
        }
    }
}

impl Default for DidWeb<HttpsConnector<HttpConnector>> {
    fn default() -> Self {
        Self::new()
    }
}

impl DidWeb<HttpsConnector<HttpConnector>> {
    /// Creates a new `DidWeb` resolver.
    pub fn new() -> DidWeb<HttpsConnector<HttpConnector>> {
        DidWeb {
            client: Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(HttpsConnector::new()),
        }
    }
}

impl<C> DidWeb<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    /// Fetches a DID document from the given URL
    async fn fetch_did_document(&self, url: Uri) -> Result<String, DidWebError> {
        let res = self.client.get(url).await?;

        if !res.status().is_success() {
            return Err(DidWebError::NonSuccessResponse(res.status()));
        }

        let body = BodyExt::collect(res.into_body()).await?;

        String::from_utf8(body.to_bytes().to_vec()).map_err(|err| err.into())
    }

    /// Fetches and parses a DID document for the given DID.
    async fn resolver_fetcher(&self, did: &str) -> Result<DIDDocument, DidWebError> {
        let (path, domain_name) = parse_did_web_url(did).map_err(|err| DidWebError::RepresentationNotSupported(err.to_string()))?;

        // Use HTTP for localhost only during testing
        let scheme = if domain_name.starts_with("localhost") {
            Scheme::HTTP
        } else {
            Scheme::HTTPS
        };

        let url = uri::Builder::new()
            .scheme(scheme)
            .authority(domain_name)
            .path_and_query(path)
            .build()
            .map_err(|err| DidWebError::RepresentationNotSupported(err.to_string()))?;

        let json_string = self.fetch_did_document(url).await?;

        let did_document = serde_json::from_str(&json_string).map_err(|err| DidWebError::RepresentationNotSupported(err.to_string()))?;

        Ok(did_document)
    }
}

/// Parses a DID Web URL and returns the path and domain name.
fn parse_did_web_url(did: &str) -> Result<(String, String), DidWebError> {
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

    path = format!("/{path}/did.json");

    Ok((path, domain_name))
}

#[async_trait]
impl<C> DIDResolver for DidWeb<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    /// Resolves a `did:web` address to a DID document.
    ///
    /// # Example
    ///
    /// ```
    /// use did_utils::methods::{DIDResolver, DidWeb, DIDResolutionOptions};
    ///
    /// # async fn example_resolve_did_web() {
    /// // create new web did resolver
    /// let did_web_resolver = DidWeb::new();
    /// let did = "did:web:example.com";
    /// // resolve the did
    /// let output = did_web_resolver.resolve(did, &DIDResolutionOptions::default()).await;
    /// # }
    /// ```
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput {
        let context = Context::SingleString(String::from("https://w3id.org/did-resolution/v1"));

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

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{body::Body, extract::Request, response::Response, routing::get};
    use serde_json::Value;
    use std::{convert::Infallible, net::SocketAddr};
    use tokio::net::TcpListener;

    async fn mock_server_handler(req: Request) -> Result<Response, Infallible> {
        const DID_JSON: &str = r#"
            {"@context": "https://www.w3.org/ns/did/v1",
            "id": "did:web:localhost",
                  "verificationMethod": [{
                  "id": "did:web:localhost#key1",
                  "type": "Ed25519VerificationKey2018",
                  "controller": "did:web:localhost",
                  "publicKeyJwk": {
                      "key_id": "ed25519-2020-10-18",
                      "kty": "OKP",
                      "crv": "Ed25519",
                      "x": "G80iskrv_nE69qbGLSpeOHJgmV4MKIzsy5l5iT6pCww"
                  }
                  }],
                  "assertionMethod": ["did:web:localhost#key1"]
            }"#;

        let response = match req.uri().path() {
            "/.well-known/did.json" | "/user/alice/did.json" => Response::new(Body::from(DID_JSON)),
            _ => Response::builder().status(404).body(Body::from("Not Found")).unwrap(),
        };

        Ok(response)
    }

    async fn create_mock_server(port: u16) -> String {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await.unwrap();

        let app = axum::Router::new().route("/.well-known/did.json", get(mock_server_handler));

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        "localhost".to_string()
    }

    #[tokio::test]
    async fn resolves_document() {
        let port = 3000;
        let host = create_mock_server(port).await;

        let formatted_string = format!("did:web:{host}%3A{port}");

        let did: &str = &formatted_string;

        let did_web_resolver = DidWeb::http();
        let output: ResolutionOutput = did_web_resolver.resolve(did, &DIDResolutionOptions::default()).await;

        let expected: Value = serde_json::from_str(
            r#"{
                "@context": "https://w3id.org/did-resolution/v1",
                "didDocument": {
                    "@context": "https://www.w3.org/ns/did/v1",
                    "assertionMethod": ["did:web:localhost#key1"],
                    "id": "did:web:localhost",
                    "verificationMethod": [
                        {
                            "controller": "did:web:localhost",
                            "id": "did:web:localhost#key1",
                            "publicKeyJwk": {
                            "crv": "Ed25519",
                            "kty": "OKP",
                            "x": "G80iskrv_nE69qbGLSpeOHJgmV4MKIzsy5l5iT6pCww"
                            },
                            "type": "Ed25519VerificationKey2018"
                        }
                    ]
                },
                "didDocumentMetadata": null,
                "didResolutionMetadata": {
                    "contentType": "application/did+ld+json"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(json_canon::to_string(&output).unwrap(), json_canon::to_string(&expected).unwrap());
    }

    use crate::methods::web::resolver;

    #[test]
    fn test_parse_did_web_url() {
        let input_1 = "did:web:w3c-ccg.github.io";
        let result_1 = resolver::parse_did_web_url(input_1);
        assert!(result_1.is_ok(), "Expected Ok, got {result_1:?}");
        let (path_1, domain_name_1) = result_1.unwrap();
        assert_eq!(domain_name_1, "w3c-ccg.github.io");
        assert_eq!(path_1, "/.well-known/did.json");

        let input_2 = "did:web:w3c-ccg.github.io:user:alice";
        let result_2 = resolver::parse_did_web_url(input_2);
        assert!(result_2.is_ok(), "Expected Ok, got {result_2:?}");
        let (path_2, domain_name_2) = result_2.unwrap();
        assert_eq!(domain_name_2, "w3c-ccg.github.io");
        assert_eq!(path_2, "/user/alice/did.json");

        let input_3 = "did:web:example.com%3A3000:user:alice";
        let result_3 = resolver::parse_did_web_url(input_3);
        assert!(result_3.is_ok(), "Expected Ok, got {result_3:?}");
        let (path_3, domain_name_3) = result_3.unwrap();
        assert_eq!(domain_name_3, "example.com:3000");
        assert_eq!(path_3, "/user/alice/did.json");
    }
}
