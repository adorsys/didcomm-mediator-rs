//! Implementation of the DID Web method as defined in [the spec](https://w3c-ccg.github.io/did-method-web/).
//! 
//! # Examples
//! 
//! ### Parse DID Web URL
//! 
//! ```rust
//! use did_utils::methods::did_web::resolver;
//! use did_utils::methods::errors::DidWebError;
//! 
//! let input_1 = "did:web:w3c-ccg.github.io";
//! let result_1 = resolver::parse_did_web_url(input_1);
//! let (path_1, domain_name_1) = result_1.unwrap();
//!
//! let input_3 = "did:web:example.com%3A3000:user:alice";
//! let result_3 = resolver::parse_did_web_url(input_3);
//! let (path_3, domain_name_3) = result_3.unwrap();
//! ```
//! 
//! ### Resolve DID Web URL
//! 
//! ```rust
//! async fn resolves_did_web_document() {
//! 
//!     use did_utils::methods::{
//!         did_web::resolver::DidWebResolver,
//!         traits::{ DIDResolutionOptions, DIDResolver, ResolutionOutput },
//!     };
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

pub mod resolver;
