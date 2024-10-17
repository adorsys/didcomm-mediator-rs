# did-endpoint

The `did-endpoint` crate provides tools and functionalities for generating and managing Decentralized Identifiers (DIDs) and web-based interactions.

## Features

- **Generates keys and forward them for DID generation:**
- **Builds and persists DID document:**
- **Validates the integrity of the persisted DID document**

## Usage

To use `did-endpoint` in your project, add the following to your **Cargo.toml**:

```toml
did-endpoint = "0.1.0"
```

### Example

Here’s a simple example of how you can generate and validate a DID document:

```rust
use did_endpoint::{didgen, validate_diddoc};

let (storage_dirpath, server_public_domain) = ("target/storage", "https://example.com");

// generate and persist a did document
didgen(&storage_dirpath, &server_public_domain)?;
// validate the generated did document
assert!(validate_diddoc(&storage_dirpath).is_ok());
```
