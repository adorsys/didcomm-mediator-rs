use serde::{ Deserialize, Serialize };

use crate::key_jwk::bytes::Bytes;
use crate::key_jwk::secret::Secret;

/// An elliptic-curve key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ec {
    /// The elliptic curve identifier.
    pub crv: EcCurves,

    /// The public x coordinate.
    pub x: Bytes,

    /// The public y coordinate.
    pub y: Bytes,

    /// The private key.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    //A secret is like the [`Bytes`] type, but can be expanded to include additional protections:
    pub d: Option<Secret>,
}

/// The elliptic curve.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EcCurves {
    /// P-256
    #[serde(rename = "P-256")]
    P256,

    /// P-384
    #[serde(rename = "P-384")]
    P384,

    /// P-521
    #[serde(rename = "P-521")]
    P521,

    /// P-256K
    #[serde(rename = "secp256k1")]
    P256K,
}
