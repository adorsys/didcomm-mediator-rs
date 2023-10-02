use crate::methods::{
    did_web::resolver::DidWebResolver,
    traits::{DIDResolutionOptions, DIDResolver, ResolutionOutput},
};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use serde_json::Value;
use std::convert::Infallible;
use std::net::SocketAddr;

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

async fn mock_server_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = match req.uri().path() {
        "/.well-known/did.json" | "/user/alice/did.json" => Response::new(Body::from(DID_JSON)),
        _ => Response::builder().status(404).body(Body::from("Not Found")).unwrap(),
    };

    Ok(response)
}

async fn create_mock_server(port: u16) -> String {
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(mock_server_handler)) });

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let server = Server::bind(&addr).serve(make_svc);

    tokio::spawn(async move {
        server.await.unwrap();
    });

    "localhost".to_string()
}

#[tokio::test]
async fn resolves_document() {
    let port = 3000;
    let host = create_mock_server(port).await;

    let formatted_string = format!("did:web:{}%3A{}", host, port);

    let did: &str = &formatted_string;

    let did_web_resolver = DidWebResolver::http();
    let output: ResolutionOutput = did_web_resolver.resolve(did, &DIDResolutionOptions::default()).await;

    let expected: Value = serde_json::from_str(
        r#"{
          "@context": "https://www.w3.org/ns/did/v1",
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
        }
        "#,
    )
    .unwrap();

    assert_eq!(json_canon::to_string(&output).unwrap(), json_canon::to_string(&expected).unwrap(),);
}
