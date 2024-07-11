//! This module provides utilities for creating and verifying proofs.

pub mod model;
pub mod traits;
pub mod eddsa_jcs_2022;

// public re-exports
pub use eddsa_jcs_2022::EdDsaJcs2022;
pub use model::Proof;
pub use traits::CryptoProof;