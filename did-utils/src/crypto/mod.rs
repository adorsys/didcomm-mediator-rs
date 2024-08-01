#![warn(missing_docs)]

//! This module contains cryptographic utilities.
//! 
//! Provides interfaces and implementations for cryptographic key management,
//! including key generation, signing, verification, and key exchange operations.
//! It supports multiple curve algorithms, including [Ed25519], [X25519], and [SHA-256] hashing.
//! 
//! [Ed25519]: https://en.wikipedia.org/wiki/EdDSA
//! [X25519]: https://en.wikipedia.org/wiki/X25519
//! [SHA-256]: https://en.wikipedia.org/wiki/SHA-2

mod ed25519;
mod errors;
mod format;
mod sha256_hash;
mod traits;
mod utils;
mod x25519;

pub use ed25519::Ed25519KeyPair;
pub use errors::Error;
pub use sha256_hash::{sha256_hash, sha256_multihash};
pub use traits::{CoreSign, Generate, KeyMaterial, BYTES_LENGTH_32, ECDH};
pub use x25519::X25519KeyPair;

/// A wrapper struct for an asymmetric key pair.
/// This struct holds a public key and an optional secret key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsymmetricKey<P, S> {
    pub(super) public_key: P,
    pub(super) secret_key: Option<S>,
}
