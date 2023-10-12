use serde::{ Deserialize, Serialize };

use crate::key::ec::Ec;
use crate::key::okp::Okp;

/// A key type that can be contained in a JWK.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "kty")]
#[non_exhaustive]
pub enum Key {
    /// An elliptic-curve key.
    Ec(Ec),

    /// A CFRG-curve key.
    Okp(Okp),
}

impl From<Ec> for Key {
    #[inline(always)]
    fn from(key: Ec) -> Self {
        Self::Ec(key)
    }
}

impl From<Okp> for Key {
    #[inline(always)]
    fn from(key: Okp) -> Self {
        Self::Okp(key)
    }
}
