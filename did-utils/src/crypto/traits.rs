// Inspired from https://github.com/decentralized-identity/did-key.rs
// We are desiging the application to support many curve algorithms.
// This module will design an interface common to all curves, so that we can change the curve
// without altering consuming modules.

#[derive(Debug)]
pub enum Error {
    SignatureError,
    InvalidKey,
    Unknown(String),
}

/// Return key material bytes
pub trait KeyMaterial {
    /// Returns the public key bytes as slice
    fn public_key_bytes(&self) -> Vec<u8>;
    /// Returns the secret key bytes as slice
    fn private_key_bytes(&self) -> Vec<u8>;
}

/// Deterministic Key Generation
pub trait Generate: KeyMaterial {
    /// Generate random key
    fn new() -> Self;
    /// Generate key deterministically using a given seed
    fn new_with_seed(seed: &[u8]) -> Self;
    /// Generate instance from existing public key
    fn from_public_key(public_key: &[u8]) -> Self;
    /// Generate instance from existing secret key
    fn from_secret_key(private_key: &[u8]) -> Self;
}

/// ECDSA Interface
pub trait CoreSign {
    /// Performs sign operation
    fn sign(&self, payload: &[u8]) -> Vec<u8>;
    /// Performs verify operation
    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error>;
}

/// ECDH Interface
pub trait ECDH {
    /// Perform key exchange operation
    fn key_exchange(&self, their_public: &Self) -> Vec<u8>;
}

