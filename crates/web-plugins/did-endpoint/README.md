# did-endpoint

The `did-endpoint` plugin crate provides a set of tools for generating and validating a DID document. It is a part of the [Didcomm Mediator](https://github.com/adorsys/didcomm-mediator-rs/) project.

## Features

- **Builds and persists DID document:**
- **Validates the integrity of the persisted DID document**

### Example

Here’s a simple example of how you can generate and validate a DID document:

```rust
use did_endpoint::{
    didgen,
    persistence::DidDocumentRepository,
};
use database::get_or_init_database;
use keystore::Keystore;

// This requires MONGODB_URI and MONGODB_DATABASE environment variables to be set.
let db = get_or_init_database();
let repository = DidDocumentRepository::from_db(&db);
let keystore = Keystore::with_mongodb();
let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").unwrap();

// Generate and persist a new DID document
didgen::didgen(&server_public_domain, &keystore, &repository)?;

// Validate the integrity of the persisted DID document
didgen::validate_diddoc(&keystore, &repository)?;
```
