//! # did-utils
//! 
//! This crate provides a set of utilities for working with Decentralized Identifiers (DIDs).
//! It includes support for cryptographic operations, DID core functionality, key management, proof handling, 
//! verifiable credentials, linked data models, and various DID methods.
//!
//! ## Modules
//!
//! - [`crypto`]: Contains cryptographic utilities for key generation, encryption, 
//!   decryption, signing, and verification.
//! - [`didcore`]: Provides core functionality for DIDs, including parsing and manipulation.
//! - [`didkit`]: Provides high-level functionality for creating and managing DIDs.
//! - [`key_jwk`]: Provides support for JSON Web Key (JWK) representations of keys.
//! - [`proof`]: Handles proof creation and verification.
//! - [`vc`]: Manages Verifiable Credentials, including their creation, signing, and verification.
//! - [`ldmodel`]: Defines Linked Data models for representing DIDs and related data.
//! - [`methods`]: Implements various DID methods.
//!
//! ## Example Usage
//!
//! Below is a simple example of how to create a DID Document:
//!
//! ```rust
//! # use did_utils::didcore::Document;
//! # use did_utils::ldmodel::Context;
//!
//! # fn main() {
//!     let context = Context::SetOfString(vec!["https://www.w3.org/ns/did/v1".to_string()]);
//!     let did_document = Document::new(context, "did:example:123456".to_string());
//!     println!("{:?}", did_document);
//! # }
//! ```
pub mod crypto;
pub mod didcore;
pub mod key_jwk;
pub mod ldmodel;
pub mod methods;
<<<<<<< Updated upstream
pub mod didkit;
=======
pub mod proof;
pub mod vc;
>>>>>>> Stashed changes
