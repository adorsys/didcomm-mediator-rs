//! This module provides cryptographic utilities and key pair structures
//! for various cryptographic algorithms, including Ed25519, X25519, and SHA-256 hashing.
//!
//! The module includes the following submodules:
//! - [`ed25519`]: Provides Ed25519 key pair generation and signature functionality.
//! - [`x25519`]: Provides X25519 key pair generation and Diffie-Hellman key exchange functionality.
//! - [`traits`]: Defines common traits for cryptographic operations.
//! - [`mod@sha256_hash`]: Provides functionality for SHA-256 hashing.
//!
//! The module also re-exports key types and utilities for easier access.
//!
//! # Example
//!
//! ```rust
//! use did_utils::crypto::{Ed25519KeyPair, X25519KeyPair,
//!                         sha256_hash,
//!                         Generate, CoreSign, ECDH};
//! 
//! // Example usage of Ed25519 key pair
//! let keypair = Ed25519KeyPair::new().unwrap();
//! let json_file = "test_resources/crypto_ed25519_test_sign_verify.json";
//! let json_data = std::fs::read_to_string(json_file).unwrap();
//! let signature = keypair.sign(json_data.as_bytes());
//! // Verify the signature
//! let verified = keypair.verify(json_data.as_bytes(), &signature.unwrap());
//! 
//! // Example usage of X25519 key pair
//! let alice_seed = b"TMwLj2p2qhcuVhaFAj3QkkJGhK6pdyKx";
//! let bob_seed = b"NWB6DbnIlewWVp5jIJOSgyX8msXNPPAL";
//! let alice = X25519KeyPair::new_with_seed(alice_seed).unwrap();
//! let bob = X25519KeyPair::new_with_seed(bob_seed).unwrap();
//! 
//! let alice_shared_secret = alice.key_exchange(&bob);
//! let bob_shared_secret = bob.key_exchange(&alice);
//! assert_eq!(alice_shared_secret, bob_shared_secret);
//! 
//! // Example usage of SHA-256 hashing
//! let hash = sha256_hash(json_file.as_bytes());
//!```

mod format;
mod utils;

pub mod errors;
pub mod ed25519;
pub mod traits;
pub mod x25519;
pub mod sha256_hash;

pub use errors::Error;
pub use traits::{Generate, CoreSign, ECDH, BYTES_LENGTH_32};
pub use ed25519::Ed25519KeyPair;
pub use x25519::X25519KeyPair;
pub use sha256_hash::sha256_hash;


/// A wrapper struct for an asymmetric key pair.
///
/// # Fields
///
/// - `public_key`: the public key of the key pair.
/// - `secret_key`: the optional private key of the key pair.
/// 
/// # Type Parameters
///
/// - `P`: The type of the public key.
/// - `S`: The type of the secret key.
pub struct AsymmetricKey<P, S> {
    pub public_key: P,
    pub secret_key: Option<S>,
}
