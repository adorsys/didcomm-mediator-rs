/// The set of errors that can occur during key operations.
#[derive(Debug)]
pub enum Error {
    CanNotComputePublicKey,
    CanNotRetrieveSignature,
    InvalidCurve,
    InvalidKeyLength,
    InvalidSecretKey,
    InvalidSeed,
    InvalidPublicKey,
    SignatureError,
    VerificationError,
    InvalidProof,
    InvalidCall(String),
    Unsupported,
    Unknown(String),
}