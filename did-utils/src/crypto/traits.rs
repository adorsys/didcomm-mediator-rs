// Inspired from https://github.com/decentralized-identity/did-key.rs
// We are desiging the application to support many curve algorithms.
// This module will design an interface common to all curves, so that we can change the curve
// without altering consuming modules.

pub const BYTES_LENGTH_32: usize = 32;

#[derive(Debug)]
pub enum Error {
    InvalidKeyLength,
    InvalidSecretKey,
    InvalidSeed,
    InvalidPublicKey,
    ConNotComputePublicKey,
    CanNotRetrieveSignature,
    SignatureError,
    VerificationError,
    InvalidProof,
    InvalidCall(String),
    Unknown(String),
}

/// Return key material bytes
pub trait KeyMaterial {
    /// Returns the public key bytes as slice
    fn public_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error>;
    /// Returns the secret key bytes as slice
    fn private_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error>;
}

/// Deterministic Key Generation
pub trait Generate: KeyMaterial {
    /// Generate random key
    fn new() -> Result<Self, Error> where Self: Sized;
    /// Generate key deterministically using a given seed
    fn new_with_seed(seed: &[u8]) -> Result<Self, Error> where Self: Sized;
    /// Generate instance from existing public key
    fn from_public_key(public_key: &[u8; BYTES_LENGTH_32]) -> Result<Self, Error> where Self: Sized;
    /// Generate instance from existing secret key
    fn from_secret_key(private_key: &[u8; BYTES_LENGTH_32]) -> Result<Self, Error> where Self: Sized;
}

/// ECDSA Interface
pub trait CoreSign {
    /// Performs sign operation
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Error>;
    /// Performs verify operation
    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error>;
}

/// ECDH Interface
pub trait ECDH {
    /// Perform key exchange operation
    fn key_exchange(&self, their_public: &Self) -> Option<Vec<u8>>;
}
