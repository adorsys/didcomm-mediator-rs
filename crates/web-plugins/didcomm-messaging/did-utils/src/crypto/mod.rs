//! This module contains cryptographic utilities.
//!
//! Provides interfaces and implementations for cryptographic key management,
//! including key generation, signing, verification, and key exchange operations.
//! It supports multiple curve algorithms, including [Ed25519], [X25519], and [SHA-256] hashing.
//!
//! [Ed25519]: https://en.wikipedia.org/wiki/EdDSA
//! [X25519]: https://en.wikipedia.org/wiki/X25519
//! [SHA-256]: https://en.wikipedia.org/wiki/SHA-2

pub(crate) mod alg;
mod ed25519;
mod errors;
mod format;
mod sha256_hash;
mod traits;
mod utils;
mod x25519;

pub use alg::Algorithm;
pub use ed25519::Ed25519KeyPair;
pub use errors::Error;
pub use format::PublicKeyFormat;
pub use sha256_hash::{sha256_hash, sha256_multihash};
pub use traits::{CoreSign, Generate, KeyMaterial, ToMultikey, ToPublic, BYTES_LENGTH_32, ECDH};
pub use x25519::X25519KeyPair;

/// A wrapper struct for an asymmetric key pair.
/// This struct holds a public key and an optional secret key.
pub struct AsymmetricKey<P, S> {
    pub public_key: P,
    pub secret_key: Option<S>,
}
