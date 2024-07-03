# did-endpoint 
## Overview
The `did-endpoint` is a robust, flexible, and extendable framework for managing Decentralized Identifiers (DIDs)

## Purpose
The DID endpoint aims to simplify the management of Decentralized Identifiers (DIDs) by providing a decentralized service that supports multiple DID methods.

## Features
- **Turns an HTTP(S) URL into a did:web id:** 
- **Generates keys and forward them for DID generation:**
- **Builds and persists DID document:**
- **Validates the integrity of the persisted diddoc:**
- **generates a verifiable presentation (VP):**

## Usage
- **Turns an HTTP(S) URL into a did:web id:** 
```rust
        //This is a function call to url_to_did_web_id with the argument "localhost:8080".
        //"localhost:8080" is the URL that you want to convert into a did:web identifier.
            url_to_did_web_id("localhost:8080")?,
        //This is the expected output, which is the did:web identifier corresponding to the given URL
            "did:web:localhost%3A8080",
```
- **Generates keys and forward them for DID generation::**
```rust
let authentication_key = Jwk {
            key: Key::Okp(Okp {
                crv: OkpCurves::Ed25519,
                x: Bytes::from(
                    String::from(
                        "d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259",
                    )
                    .into_bytes(),
                ),
                d: None,
            }),
            prm: Parameters::default(),
        };

let assertion_key = Jwk {
        key: Key::Okp(Okp {
            crv: OkpCurves::Ed25519,
            x: Bytes::from(
                String::from(
                    "d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259",
                )
                .into_bytes(),
            ),
                d: None,
            }),
            prm: Parameters::default(),
        };

let agreement_key = Jwk {
        key: Key::Okp(Okp {
            crv: OkpCurves::X25519,
            x: Bytes::from(
                String::from(
                    "d75a980182b10ab2463c5b1be1b4d97e06ec21ebac8552059996bd962d77f259",
                )
                .into_bytes(),
            ),
            d: None,
        }),
        prm: Parameters::default(),
    }
let diddoc = gen_diddoc(
            &storage_dirpath,
            &server_public_domain,
            authentication_key.clone(),
            assertion_key.clone(),
            agreement_key.clone(),
        )
```
- **Builds and persists DID document:**
```rust
expected_verification_methods = vec![
        VerificationMethod {
            id: "did:web:example.com#keys-1".to_string(),
            public_key: Some(KeyFormat::Jwk(authentication_key)),
            ..VerificationMethod::new(
                "did:web:example.com#keys-1".to_string(),
                String::from("JsonWebKey2020"),
                "did:web:example.com".to_string(),
            )
        },
        VerificationMethod {
            id: "did:web:example.com#keys-2".to_string(),
            public_key: Some(KeyFormat::Jwk(assertion_key)),
            ..VerificationMethod::new(
                "did:web:example.com#keys-2".to_string(),
                String::from("JsonWebKey2020"),
                "did:web:example.com".to_string(),
            )
        },
        VerificationMethod {
            id: "did:web:example.com#keys-3".to_string(),
            public_key: Some(KeyFormat::Jwk(agreement_key)),
            ..VerificationMethod::new(
                "did:web:example.com#keys-3".to_string(),
                String::from("JsonWebKey2020"),
                "did:web:example.com".to_string(),
            )
        },
    ]
```
- **Validates the integrity of the persisted diddoc:**
```rust
(storage_dirpath, server_public_domain) = setup();

        didgen(&storage_dirpath, &server_public_domain).unwrap();
        assert!(validate_diddoc(&storage_dirpath).is_ok());

        cleanup(&storage_dirpath);
```
- **generates a verifiable presentation (VP):**
```rust
        // Generate test-restricted did.json
        let (storage_dirpath, expected_diddoc) = setup_ephemeral_diddoc();

        let app = routes();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/.well-known/did/pop.json?challenge={}",
                        uuid::Uuid::new_v4()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let vp: VerifiablePresentation = serde_json::from_slice(&body).unwrap();

        let vc = vp.verifiable_credential.get(0).unwrap();
        let diddoc = serde_json::from_value(json!(vc.credential_subject)).unwrap();

        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),
            json_canon::to_string(&expected_diddoc).unwrap()
        );

        let Some(proofs) = &vp.proof else { panic!("Verifiable presentation carries no proof") };
        let Proofs::SetOfProofs(proofs) = proofs else { unreachable!() };
        for proof in proofs {
            let pubkey = resolve_vm_for_public_key(&diddoc, &proof.verification_method)
                .expect("ResolutionError");
            let verifier = EdDsaJcs2022 {
                proof: proof.clone(),
                key_pair: pubkey.try_into().expect("Failure to convert to KeyPair"),
                proof_value_codec: None,
            };

            assert!(verifier.verify(json!(vp)).is_ok());
        }

        cleanup(&storage_dirpath);
```
