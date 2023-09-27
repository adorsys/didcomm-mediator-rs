use crate::methods::{
    did_web::{self, resolver::DidWebResolver},
    traits::{DIDResolutionOptions, DIDResolver, ResolutionOutput},
};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::convert::Infallible;
use std::net::SocketAddr;

const DID_DOCUMENT: &'static str = r##"
{
  "@context": [
    "https://www.w3.org/ns/did/v1",
    "https://w3id.org/security/suites/jws-2020/v1"
  ],
  "id": "did:web:example.com",
  "verificationMethod": [
    {
      "id": "did:web:example.com#key-0",
      "type": "JsonWebKey2020",
      "controller": "did:web:example.com",
      "publicKeyJwk": {
        "kty": "OKP",
        "crv": "Ed25519",
        "x": "0-e2i2_Ua1S5HbTYnVB0lj2Z2ytXu2-tYmDFf8f5NjU"
      }
    },
    {
      "id": "did:web:example.com#key-1",
      "type": "JsonWebKey2020",
      "controller": "did:web:example.com",
      "publicKeyJwk": {
        "kty": "OKP",
        "crv": "X25519",
        "x": "9GXjPGGvmRq9F6Ng5dQQ_s31mfhxrcNZxRGONrmH30k"
      }
    },
    {
      "id": "did:web:example.com#key-2",
      "type": "JsonWebKey2020",
      "controller": "did:web:example.com",
      "publicKeyJwk": {
        "kty": "EC",
        "crv": "P-256",
        "x": "38M1FDts7Oea7urmseiugGW7tWc3mLpJh6rKe7xINZ8",
        "y": "nDQW6XZ7b_u2Sy9slofYLlG03sOEoug3I0aAPQ0exs4"
      }
    }
  ],
  "authentication": [
    "did:web:example.com#key-0",
    "did:web:example.com#key-2"
  ],
  "assertionMethod": [
    "did:web:example.com#key-0",
    "did:web:example.com#key-2"
  ],
  "keyAgreement": [
    "did:web:example.com#key-1",
    "did:web:example.com#key-2"
  ]
}"##;

async fn mock_server_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = match req.uri().path() {
        "/.well-known/did.json" => Response::new(Body::from(DID_DOCUMENT)),
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

fn add(a: i32, b: i32) -> i32 {
    a + b
}

const did: &str = "did:web:example.com";

#[tokio::test]
async fn resolves_document() {
    let port = 3000;
    let host = create_mock_server(port).await;

    let did_web_resolver = DidWebResolver::https();

    let output: ResolutionOutput = did_web_resolver.resolve(did, &DIDResolutionOptions::default()).await;

    println!("Output: {:?}", output);

    assert_eq!(add(2, 3), 5);
}
