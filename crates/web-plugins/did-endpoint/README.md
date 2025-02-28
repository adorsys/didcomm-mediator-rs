# did-endpoint

The `did-endpoint` plugin crate provides a set of tools for generating and validating a DID document. It is a part of the [Didcomm Mediator](https://github.com/adorsys/didcomm-mediator-rs/) project.

## Features

- **Builds and persists DID document:**
- **Validates the integrity of the persisted DID document**

### Example

Here’s a simple example of how you can generate and validate a DID document:

```rust
use did_endpoint::{didgen, validate_diddoc};
use filesystem::{FileSystem, StdFileSystem};
use keystore::KeyStore;

let storage_dirpath = std::env::var("STORAGE_DIRPATH").unwrap(),
let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").unwrap();

let mut filesystem = filesystem::StdFileSystem;
let keystore = keystore::KeyStore::with_mongodb();

// Generate and persist a new DID document
didgen::didgen(
    storage_dirpath,
    server_public_domain,
    &keystore,
    &mut filesystem,
)?;

// Validate the integrity of the persisted DID document
didgen::validate_diddoc(storage_dirpath, &keystore, &mut filesystem)?;
```
