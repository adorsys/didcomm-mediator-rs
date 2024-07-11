use serde_json::Value;

use crate::crypto::errors::Error;

use super::model::Proof;

/// A trait to be implemented by every crypto suite
pub trait CryptoProof {
    /// Create the proof value and add it to the proof object.
    /// 
    /// The payload is the data to be signed without any proof entry.
    /// Caller must make sure all existing proofs are removed prior to passing
    /// the payload to this function.
    /// 
    /// Returns the proof object with the proof value added.
    fn proof(&self, payload: Value) -> Result<Proof, Error>;

    /// Verifies that this proof is authenticates with the payload.
    /// 
    /// The payload is the data to be verified without any proof entry.
    /// Caller must make sure all existing proofs are removed prior to passing
    /// the payload to this function.
    fn verify(&self, payload: Value) -> Result<(), Error>;
}