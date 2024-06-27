//! The did:key method is a non-registry approach to DID Methods based on expanding
//! a cryptographic public key into a DID Document. This approach provides the
//! simplest possible implementation of a DID Method that is able to achieve many,
//! but not all, of the benefits of utilizing DIDs.
//!
//! See https://w3c-ccg.github.io/did-method-key

pub mod method;
pub mod resolver;

pub use method::DIDKeyMethod;
