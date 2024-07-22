//! This module provides types and utilities for handling JSON Web Keys (JWKs).
//! 
//! It includes support for various key types, secure serialization and deserialization, and encoding schemes.
//!
//! ## Submodules
//!
//! - [`bytes`]:  Contains utilities for handling byte sequences, including secure serialization and deserialization using Base64 encoding.
//! - [`ec`]:     Provides support for working with elliptic-curve keys.
//! - [`jwk`]:    Contains types and utilities for working with JSON Web Keys (JWKs).
//! - [`key`]:    Provides generic key types and associated utilities used in JSON Web Keys (JWKs).
//! - [`oct`]:    Provides support for working with octet sequence keys.
//! - [`okp`]:    Provides support for working with Octet Key Pairs (OKP).
//! - [`prm`]:    Defines parameter-related types for keys and cryptographic operations.
//! - [`rsa`]:    Provides support for working with RSA keys.
//! - [`secret`]: Provides utilities for working with secrets securely.
//! 
//! //! # Examples
//!
//! ```no run
//! # use did_utils::key_jwk::Bytes;
//! # use base64ct::Base64UrlUnpadded;
//!
//! // Creating a Bytes instance from a vector
//! let data = vec![1, 2, 3, 4];
//! let bytes: Bytes<Vec<u8>, Base64UrlUnpadded> = Bytes::from(data);
//!
//! // Serializing to a base64 string
//! let serialized = serde_json::to_string(&bytes)?;
//!
//! // Deserializing from a base64 string
//! let deserialized: Bytes<Vec<u8>, Base64UrlUnpadded> = serde_json::from_str(&serialized)?;
//! ```

mod bytes;
mod ec;
mod jwk;
mod key;
mod oct;
mod okp;
mod prm;
mod rsa;
mod secret;

// Re-exports
pub use bytes::Bytes;
pub use ec::{Ec, EcCurves};
pub use jwk::{Jwk, JwkSet};
pub use key::Key;
pub use oct::Oct;
pub use okp::{Okp, OkpCurves};
pub use prm::Parameters;
pub use rsa::Rsa;
pub use secret::Secret;