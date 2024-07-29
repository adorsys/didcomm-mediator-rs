// Inspired from https://github.com/decentralized-identity/did-key.rs
// We are desiging the application to support many curve algorithms.
// This module will design an interface common to all curves, so that we can change the curve
// without altering consuming modules.

use super::errors::Error;

/// The length of a 32-byte key material.
pub const BYTES_LENGTH_32: usize = 32;

/// A trait for types that hold key material bytes.
pub trait KeyMaterial {
    /// Returns the bytes of the public key.
    fn public_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error>;

    /// Returns the bytes of the private key.
    fn private_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error>;
}

/// A trait for types that support deterministic key generation.
pub trait Generate: KeyMaterial {
    /// Generates a new random key pair.
    fn new() -> Result<Self, Error>
    where
        Self: Sized;

    /// Generates a new key pair with a given seed.
    ///
    /// If the seed is empty or invalid, a random seed will be generated.
    fn new_with_seed(seed: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Generates a new key pair from a public key.
    fn from_public_key(public_key: &[u8; BYTES_LENGTH_32]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Generates a new key pair from a secret key.
    ///
    /// A public key will be generated from the secret key.
    fn from_secret_key(private_key: &[u8; BYTES_LENGTH_32]) -> Result<Self, Error>
    where
        Self: Sized;
}

/// A trait for types that support signing and verification operations.
pub trait CoreSign {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Error>;
    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error>;
}

/// A trait for types that support ECDH key exchange operations.
pub trait ECDH {
    fn key_exchange(&self, their_public: &Self) -> Option<Vec<u8>>;
}
