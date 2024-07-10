//! A collection of modules for DID resolution and related utilities.
//!
//! This crate provides functionality for resolving Decentralized Identifiers (DIDs)
//! using different DID methods. The main modules include:
//! - [`errors`]: Defines error types used across the crate.
//! - [`traits`]: Defines traits for DID resolution.
//! - [`did_key`]: Implements resolution for [`did:key`] method.
//! - [`did_web`]: Implements resolution for [`did:web`] method.
//! 
//! [`did:key`]: https://w3c-ccg.github.io/did-method-key/
//! [`did:web`]: https://w3c-ccg.github.io/did-method-web/
//! 
//! # Examples
//! 
//! ### did:key usage
//! 
//! ```rust
//! use did_utils::methods::{traits::*, DIDKeyMethod};
//! 
//! async fn test_did_key() {
//!     let did_method = DIDKeyMethod {
//!         enable_encryption_key_derivation: true,
//!         ..Default::default()
//!     };
//!     let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
//!     let output = did_method.resolve(did, &DIDResolutionOptions::default()).await;
//!     assert!(output.did_document.is_some());
//! }
//! ```
//! 
//! ### did:web usage
//! 
//! ```rust
//! async fn resolves_did_web_document() {
//! 
//!     use did_utils::methods::{traits::*, DidWebResolver};
//! 
//!     let port = 3000;
//!     let host = "localhost";
//! 
//!     let formatted_string = format!("did:web:{}%3A{}", host.to_string(), port);
//! 
//!     let did: &str = &formatted_string;
//! 
//!     let did_web_resolver = DidWebResolver::http();
//!     let output: ResolutionOutput = did_web_resolver.resolve(
//!         did,
//!         &DIDResolutionOptions::default()
//!     ).await;
//! }
//! ```

pub mod errors;
pub mod traits;

pub mod did_key;
pub mod did_web;

pub(crate) mod utils;

pub use errors::*;
pub use traits::*;
pub use did_web::resolver::DidWebResolver;
pub use did_key::method::DIDKeyMethod;
