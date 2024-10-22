use serde::{Deserialize, Serialize};

use crate::jwk::{ec::Ec, oct::Oct, okp::Okp, rsa::Rsa};

/// A key type that can be contained in a JWK.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "kty")]
#[non_exhaustive]
pub enum Key {
    /// An elliptic-curve key.
    Ec(Ec),

    /// An RSA key.
    Rsa(Rsa),

    /// A symmetric key.
    #[serde(rename = "oct")]
    Oct(Oct),

    /// A CFRG-curve key.
    Okp(Okp),
}

impl From<Ec> for Key {
    #[inline(always)]
    fn from(key: Ec) -> Self {
        Self::Ec(key)
    }
}

impl From<Rsa> for Key {
    #[inline(always)]
    fn from(key: Rsa) -> Self {
        Self::Rsa(key)
    }
}

impl From<Oct> for Key {
    #[inline(always)]
    fn from(key: Oct) -> Self {
        Self::Oct(key)
    }
}

impl From<Okp> for Key {
    #[inline(always)]
    fn from(key: Okp) -> Self {
        Self::Okp(key)
    }
}
