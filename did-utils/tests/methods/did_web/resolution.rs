#![allow(unused_imports)]
use did_utils::methods::{
    did_web::resolver::DidWebResolver,
    { DIDResolutionOptions, DIDResolver, ResolutionOutput },
};

use hyper::{ service::{ make_service_fn, service_fn }, Body, Request, Response, Server };

use serde_json::Value;
use std::convert::Infallible;
use std::net::SocketAddr;

#[allow(dead_code)]
async fn mock_server_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    const DID_JSON: &str =
        r#"
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

#[allow(dead_code)]
async fn create_mock_server(port: u16) -> String {
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(mock_server_handler))
    });

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
    let output: ResolutionOutput = did_web_resolver.resolve(
        did,
        &DIDResolutionOptions::default()
    ).await;

    let expected: Value = serde_json
        ::from_str(
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
        }
        "#
        )
        .unwrap();

    assert_eq!(json_canon::to_string(&output).unwrap(), json_canon::to_string(&expected).unwrap());
}

use did_utils::methods::did_web::resolver;
use did_utils::methods::errors::DidWebError;

#[test]
fn test_parse_did_web_url() {
    let input_1 = "did:web:w3c-ccg.github.io";
    let result_1 = resolver::parse_did_web_url(input_1);
    assert!(result_1.is_ok(), "Expected Ok, got {:?}", result_1);
    let (path_1, domain_name_1) = result_1.unwrap();
    assert_eq!(domain_name_1, "w3c-ccg.github.io");
    assert_eq!(path_1, "/.well-known/did.json");

    let input_2 = "did:web:w3c-ccg.github.io:user:alice";
    let result_2 = resolver::parse_did_web_url(input_2);
    assert!(result_2.is_ok(), "Expected Ok, got {:?}", result_2);
    let (path_2, domain_name_2) = result_2.unwrap();
    assert_eq!(domain_name_2, "w3c-ccg.github.io");
    assert_eq!(path_2, "/user/alice/did.json");

    let input_3 = "did:web:example.com%3A3000:user:alice";
    let result_3 = resolver::parse_did_web_url(input_3);
    assert!(result_3.is_ok(), "Expected Ok, got {:?}", result_3);
    let (path_3, domain_name_3) = result_3.unwrap();
    assert_eq!(domain_name_3, "example.com:3000");
    assert_eq!(path_3, "/user/alice/did.json");
}
