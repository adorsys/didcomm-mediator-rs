# did-utils

A Rust library for implementing reusable utility code for DID-based applications.

## Features

* Manipulate JSON DID documents.
* Create keys.
* Sign, verify, encrypt, and decrypt DID documents.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
did-utils = "0.1"
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

## Documentation

The documentation for the library is available here: https://docs.rs/did-utils/

## Contributors

* Bard
* [Your name]

## License

The library is licensed under the Apache License.
