/// The set of errors that can occur during key operations.
#[derive(Debug)]
pub enum Error {
    /// Can not compute public key
    CanNotComputePublicKey,
    /// Can not retrieve signature
    CanNotRetrieveSignature,
    /// Invalid curve
    InvalidCurve,
    /// Invalid key length
    InvalidKeyLength,
    /// Invalid secret key
    InvalidSecretKey,
    /// Invalid seed
    InvalidSeed,
    /// Invalid public key
    InvalidPublicKey,
    /// Error while signing
    SignatureError,
    /// Error while verifying
    VerificationError,
    /// Invalid proof
    InvalidProof,
    /// Invalid call
    InvalidCall(String),
    /// Unsupported algorithm
    Unsupported,
    /// Unknown error
    Unknown(String),
}