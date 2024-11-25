[package]
name = "did-utils" 
version = "0.1.0" 
authors = ["adorsys GmbH Co. KG"] 
license = "Apache-2.0"
description = "A Rust library for implementing reusable utility code for DID-based applications"
repository = "https://github.com/adorsys/didcomm-mediator-rs/tree/main/crates/web-plugins/didcomm-messaging/did-utils"
keywords = ["did-utils", "DIDComm Messaging","DIDComm", "DIDComm Mediator", "DIDComm Mediation", "Decentralized Identity", "Rust Mediator"]
categories = ["cryptography", "decentralized-systems"]
edition = "2021"

## Features

* Manipulate JSON DID documents.
* Create keys.
* Sign, verify, encrypt, and decrypt DID documents.

## Installation

```rust
cargo install did-utils
```

## Usage

```rust
use did_utils::*;

// Create a DID document.
let did_document = DidDocument::new(
    "did:example:1234567890",
    "My DID",
    "My public key",
);

// Sign the DID document.
let signature = did_document.sign(&my_private_key);

// Verify the signature of the DID document.
let is_valid = did_document.verify_signature(&signature);

// Encrypt the DID document.
let encrypted_did_document = did_document.encrypt(&my_public_key);

// Decrypt the encrypted DID document.
let decrypted_did_document = encrypted_did_document.decrypt(&my_private_key);
```

## Dependencies

* serde
* sha2
* x25519-dalek

## Documentation

The documentation for the library is available here: https://docs.rs/did-utils/

