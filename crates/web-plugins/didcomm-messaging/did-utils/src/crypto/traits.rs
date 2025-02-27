// Inspired from https://github.com/decentralized-identity/did-key.rs
// We are desiging the application to support many curve algorithms.
// This module will design an interface common to all curves, so that we can change the curve
// without altering consuming modules.

use super::errors::Error;

/// The length of a 32-byte key material.
pub const BYTES_LENGTH_32: usize = 32;

/// A trait for types that can be converted to a multikey string.
pub trait ToMultikey {
    /// Converts keypair into its multikey string
    fn to_multikey(&self) -> String;
}

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
    /// If the seed is empty or invalid, a random seed will be generated.
    fn new_with_seed(seed: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Generates a new key pair from a public key.
    fn from_public_key(public_key: &[u8; BYTES_LENGTH_32]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Generates a new key pair from a secret key.
    /// A public key will be generated from the secret key.
    fn from_secret_key(private_key: &[u8; BYTES_LENGTH_32]) -> Result<Self, Error>
    where
        Self: Sized;
}

/// A trait for types that support signing and verification operations.
pub trait CoreSign {
    /// Signs the payload with the key pair.
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Error>;

    /// Verifies the signature of the payload with the key pair.
    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error>;
}

/// A trait for types that support ECDH key exchange operations.
pub trait ECDH {
    /// Performs ECDH key exchange with the given public key.
    fn key_exchange(&self, their_public: &Self) -> Option<Vec<u8>>;
}

/// A trait for converting a key to its public counterpart.
pub trait ToPublic {
    /// Converts the key to its public counterpart.
    ///
    /// This method returns a new instance of the key with any private information removed.
    /// The returned key contains only the public key components.
    fn to_public(&self) -> Self;
}
