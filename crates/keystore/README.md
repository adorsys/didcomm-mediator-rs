# Keystore Crate

The `keystore` crate is a utility library for managing cryptographic secrets. It is used in the [Didcomm Mediator](https://github.com/adorsys/didcomm-mediator-rs/) to store and retrieve cryptographic keys for DIDcomm interactions.

## Usage

This crate is internal to the [Didcomm Mediator](https://github.com/adorsys/didcomm-mediator-rs/). Below is an example of interacting with the keystore:

```rust
use keystore::{KeyStore, Secrets};
use mongodb::bson::{doc, Bson, Document};
use did_utils::jwk::Jwk;

// Initialize the keystore
let keystore = KeyStore::get();

let jwk: Jwk = serde_json::from_str(
    r#"{
        "kty": "OKP",
        "crv": "X25519",
        "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ",
        "d": "0A8SSFkGHg3N9gmVDRnl63ih5fcwtEvnQu9912SVplY"
    }"#,
)
.unwrap();

// Store a secret
let secret = Secrets {
    id: Some(ObjectId::new()),
    kid: "key-1".to_string(),
    secret_material: jwk,
};
keystore.store(secret).await?;

// Retrieve a secret by ID
let secret = keystore.find_one(doc! {"kid": "key-1"}).await?;

// Delete a secret by ID
keystore.delete_one(secret.id.unwrap()).await?;
```
