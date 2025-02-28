# Keystore Crate

The `keystore` crate is a utility library for managing cryptographic secrets. It is used in the [Didcomm Mediator](https://github.com/adorsys/didcomm-mediator-rs/) to securely store, retrieve and delete cryptographic keys for DIDcomm interactions.

## Usage

This crate is internal to the [Didcomm Mediator](https://github.com/adorsys/didcomm-mediator-rs/). Below is an example of interacting with the keystore:

```rust
use keystore::KeyStore;
use did_utils::jwk::Jwk;

// Create a new AWS KMS client
let config = aws_config::load_from_env().await;
let client = aws_sdk_kms::Client::new(&config);
let key_id = "test-key".to_string();

// Initialize the keystore
let keystore = KeyStore::with_aws_kms(client, key_id);

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
keystore.store("key-1", &jwk).await?;

// Retrieve a secret by ID
let secret: Option<Jwk> = keystore.retrieve("key-1").await?;

// Delete a secret by ID
keystore.delete("key-1").await?;
```
