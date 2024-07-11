//! Traits for cryptographic operations.

// Inspired from https://github.com/decentralized-identity/did-key.rs
// We are desiging the application to support many curve algorithms.
// This module will design an interface common to all curves, so that we can change the curve
// without altering consuming modules.

use super::errors::Error;

/// The length of a 32-byte key material.
pub const BYTES_LENGTH_32: usize = 32;

/// A trait for types that hold key material bytes.
pub trait KeyMaterial {
    /// Returns the public key bytes as a slice.
    ///
    /// Returns a `Result` containing the public key bytes, or an `Error` if the operation fails.
    
    fn public_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error>;
    /// Returns the secret key bytes as a slice.
    ///
    /// Returns a `Result` containing the secret key bytes, or an `Error` if the operation fails.
    fn private_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error>;
}

/// A trait for types that support deterministic key generation.
pub trait Generate: KeyMaterial {
    /// Generates a new random key.
    ///
    /// Returns a `Result` containing the new key, or an `Error` if the operation fails.
    fn new() -> Result<Self, Error> where Self: Sized;

    /// Generates a new key deterministically using the given seed.
    ///
    /// Returns a `Result` containing the new key, or an `Error` if the operation fails.
    fn new_with_seed(seed: &[u8]) -> Result<Self, Error> where Self: Sized;

    /// Generates a new instance from an existing public key.
    ///
    /// Returns a `Result` containing the new instance, or an `Error` if the operation fails.
    fn from_public_key(public_key: &[u8; BYTES_LENGTH_32]) -> Result<Self, Error> where Self: Sized;

    /// Generates a new instance from an existing secret key.
    ///
    /// Returns a `Result` containing the new instance, or an `Error` if the operation fails.
    fn from_secret_key(private_key: &[u8; BYTES_LENGTH_32]) -> Result<Self, Error> where Self: Sized;
}

/// A trait for types that support ECDSA operations.
pub trait CoreSign {
    /// Performs a sign operation.
    ///
    /// Returns a `Result` containing the signature, or an `Error` if the operation fails.
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Error>;

    /// Performs a verify operation.
    ///
    /// Returns a `Result` containing `()`, or an `Error` if the operation fails.
    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error>;
}

/// A trait for types that support ECDH key exchange operations.
pub trait ECDH {
    /// Performs a key exchange operation.
    ///
    /// Returns an `Option` containing the shared secret, or `None` if the operation fails.
    fn key_exchange(&self, their_public: &Self) -> Option<Vec<u8>>;
}
