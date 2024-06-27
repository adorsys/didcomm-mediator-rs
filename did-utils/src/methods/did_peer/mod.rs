//! The did:peer DID method is designed to be used independent of any central source of truth,
//! and is intended to be cheap, fast, scalable, and secure. It is suitable for most private
//! relationships between people, organizations, and things.
//!
//! See https://identity.foundation/peer-did-method-spec/

pub mod error;
pub mod method;
pub mod resolver;
pub mod util;

pub use method::DIDPeerMethod;
