# did-endpoint 
## Overview
The `did-endpoint` is a robust, flexible, and extendable framework for managing Decentralized Identifiers (DIDs)

## Purpose
The DID endpoint aims to simplify the management of Decentralized Identifiers (DIDs) by providing a decentralized service that supports multiple DID methods.

## Features 
- **Turns an HTTP(S) URL into a did:web id:** 
- **Generates keys and forward them for DID generation:**
- **Builds and persists DID document:**
- **Proof of Possession:**

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
let expected_verification_methods = vec![
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
- ## **Proof of Possession:**
- **Challenge Handling:**
    Retrieves a challenge from incoming query parameters to initiate PoP.
- Key Store Retrieval:

    Fetches cryptographic keys from a specified storage directory (STORAGE_DIRPATH) to sign and verify proofs.
- DID Document and Verification Methods:

    Loads the DID document and its associated verification methods, which contain public keys for cryptographic operations.
- Verifiable Credential (VC) Construction:

    Constructs a Verifiable Credential (VC) using the DID document, indicating it as a type of Verifiable Credential and DID Document.
- Verifiable Presentation (VP) Creation: 

    Constructs a Verifiable Presentation (VP) containing the VC and other necessary metadata, such as context and ID.
- Proof of Possession Generation:

    Generates proofs of possession for each verification method listed in the DID document.
    Uses cryptographic keys to sign the challenge and embeds these proofs into the VP.
- Output:

    Returns the final VP with embedded proofs as a JSON response

