//! A collection of methods for DID resolution and related utilities.
//!
//! This module provides functionality for creating and resolving Decentralized Identifiers (DIDs)
//! using different DID methods including [did:key], [did:web], and [did:peer].
//! 
//! # DID Methods
//! 
//! The following DID methods are currently supported:
//! 
//! ## did:key
//! 
//! The [did:key] method is the simplest possible implementation of a DID Method that is able to achieve many,
//! but not all, of the benefits of utilizing DIDs.
//! 
//! ## did:web
//! 
//! The [did:web] is a DID method that uses the web domain's existing reputation to create and manage DIDs.
//! 
//! ## did:peer
//! 
//! The [did:peer] DID method is designed to be used independent of any central source of truth,
//! and is intended to be cheap, fast, scalable, and secure. It is suitable for most private
//! relationships between people, organizations, and things.
//! 
//! [did:key]: https://w3c-ccg.github.io/did-method-key/
//! [did:web]: https://w3c-ccg.github.io/did-method-web/
//! [did:peer]: https://identity.foundation/peer-did-method-spec/
//! 
//! # Basic Usage
//! 
//! Basic `did:key` resolution example
//! 
//! ```
//! # use did_utils::methods::{DIDResolver, DidKey};
//! # use did_utils::methods::DIDResolutionOptions;
//! #
//! # async fn test_did_key() {
//!     let did_key_resolver = DidKey::new();
//!     let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
//!     let output = did_key_resolver.resolve(did, &DIDResolutionOptions::default()).await;
//! # }
//! ```
//! 
//! An example demonstrating a basic usage of `did:web`
//! 
//! ```
//! # use did_utils::methods::{DIDResolver, DidWeb};
//! # use did_utils::methods::DIDResolutionOptions;
//! #
//! # async fn resolves_did_web_document() {
//!     let port = 3000;
//!     let host = "localhost";
//! 
//!     let formatted_string = format!("did:web:{}%3A{}", host.to_string(), port);
//!     let did: &str = &formatted_string;
//! 
//!     let did_web_resolver = DidWeb::new();
//!     let output = did_web_resolver.resolve(did, &DIDResolutionOptions::default()).await;
//! # }
//! ```
//! 
//! An example demonstrating a basic usage of `did:peer`
//! 
//! ```
//! # use did_utils::methods::{DIDResolver, DidPeer};
//! # use did_utils::methods::DIDResolutionOptions;
//! #
//! # async fn resolves_did_peer_document() {
//!     let did = "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
//! 
//!     let did_peer_resolver = DidPeer::new();
//!     let output = did_peer_resolver.resolve(did, &DIDResolutionOptions::default()).await;
//! # }
//! ```

mod did_key;
mod did_peer;
mod did_web;
mod traits;
mod errors;
mod common;
mod utils;
mod resolution;

// Re-exported items
pub use errors::{DIDResolutionError, DidWebError, ParsingErrorSource};
pub use traits::{DIDMethod, DIDResolver};
pub use did_web::resolver::DidWeb;
pub use did_key::method::DidKey;
pub use did_peer::method::{DidPeer, Purpose, PurposedKey};
pub use common::{PublicKeyFormat, Algorithm, ToMultikey};
pub use resolution::*;
