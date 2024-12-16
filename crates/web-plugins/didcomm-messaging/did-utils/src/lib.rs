/*! # did-utils

This library provides a set of utilities for working with Decentralized Identifiers (DIDs).
It includes support for cryptographic operations, DID core functionality, key management, proof handling,
verifiable credentials, linked data models, and various DID methods.

## Features

- **Cryptographic Operations**: Comprehensive support for cryptographic operations, including key management and digital signatures.
- **DID Support**: Comprehensive support for various DID methods, enabling decentralized identity management.
- **Verifiable Credentials**: Tools for creating, managing, and verifying verifiable credentials.

*/
pub mod crypto;
pub mod didcore;
pub mod didkit;
pub mod jwk;
pub mod ldmodel;
pub mod methods;
pub mod proof;
pub mod vc;
