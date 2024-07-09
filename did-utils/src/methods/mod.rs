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

pub mod errors;
pub mod traits;

pub mod did_key;
pub mod did_web;

pub(crate) mod utils;

pub use errors::*;
pub use traits::*;
pub use did_web::resolver::DidWebResolver;
pub use did_key::method::*;
