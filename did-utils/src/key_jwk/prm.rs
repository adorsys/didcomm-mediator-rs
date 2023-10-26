use serde::{Deserialize, Serialize};

use super::secret::Secret;
extern crate alloc;
use super::Bytes;
use alloc::{boxed::Box, collections::BTreeSet, string::String, vec::Vec};
use base64ct::Base64;
use core::fmt;

/// A symmetric octet key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Oct {
    /// The symmetric key.
    pub k: Secret,
}

/// JWK parameters unrelated to the key implementation
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameters {
    /// The algorithm used with this key.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub alg: Option<Algorithm>,

    /// The key identifier.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kid: Option<String>,

    /// The key class (called `use` in the RFC).
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "use")]
    pub cls: Option<Class>,

    /// The key operations (called `key_ops` in the RFC).
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "key_ops")]
    pub ops: Option<BTreeSet<Operations>>,

    /// The URL of the X.509 certificate associated with this key.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[cfg(feature = "url")]
    pub x5u: Option<url::Url>,

    /// The X.509 certificate associated with this key.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub x5c: Option<Vec<Bytes<Box<[u8]>, Base64>>>, // base64, not base64url

    /// The X.509 thumbprint associated with this key.
    #[serde(flatten)]
    pub x5t: Thumbprint,
}

impl<T: Into<Algorithm>> From<T> for Parameters {
    fn from(value: T) -> Self {
        let alg = Some(value.into());

        let cls = match alg {
            Some(Algorithm::Signing(..)) => Some(Class::Signing),
            _ => None,
        };

        Self {
            alg,
            cls,
            ..Default::default()
        }
    }
}

/// Key Class (i.e. `use` in the RFC)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Class {
    #[serde(rename = "enc")]
    Encryption,

    #[serde(rename = "sig")]
    Signing,
}

/// Key operations (i.e. `key_use` in the RFC)
// NOTE: Keep in lexicographical order.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Operations {
    Decrypt,
    DeriveBits,
    DeriveKey,
    Encrypt,
    Sign,
    UnwrapKey,
    Verify,
    WrapKey,
}

/// An X.509 thumbprint.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thumbprint {
    /// An X.509 thumbprint (SHA-1).
    #[serde(skip_serializing_if = "Option::is_none", rename = "x5t", default)]
    pub s1: Option<Bytes<[u8; 20]>>,

    /// An X.509 thumbprint (SHA-2 256).
    #[serde(skip_serializing_if = "Option::is_none", rename = "x5t#S256", default)]
    pub s256: Option<Bytes<[u8; 32]>>,
}

/// Possible types of algorithms that can exist in an "alg" descriptor.
///
/// Currently only signing algorithms are represented.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Algorithm {
    /// Algorithms used for digital signatures and MACs
    Signing(Signing),
}

impl From<Signing> for Algorithm {
    #[inline(always)]
    fn from(alg: Signing) -> Self {
        Self::Signing(alg)
    }
}

/// Algorithms used for signing, as defined in [RFC7518] section 3.1.
///
/// [RFC7518]: https://www.rfc-editor.org/rfc/rfc7518
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Signing {
    /// EdDSA signature algorithms (Optional)
    #[serde(rename = "EdDSA")]
    EdDsa,

    /// ECDSA using P-256 and SHA-256 (Recommended+)
    Es256,

    /// ECDSA using secp256k1 curve and SHA-256 (Optional)
    Es256K,

    /// ECDSA using P-384 and SHA-384 (Optional)
    Es384,

    /// ECDSA using P-521 and SHA-512 (Optional)
    Es512,

    /// HMAC using SHA-256 (Required)
    Hs256,

    /// HMAC using SHA-384 (Optional)
    Hs384,

    /// HMAC using SHA-512 (Optional)
    Hs512,

    /// RSASSA-PSS using SHA-256 and MGF1 with SHA-256 (Optional)
    Ps256,

    /// RSASSA-PSS using SHA-384 and MGF1 with SHA-384 (Optional)
    Ps384,

    /// RSASSA-PSS using SHA-512 and MGF1 with SHA-512 (Optional)
    Ps512,

    /// RSASSA-PKCS1-v1_5 using SHA-256 (Recommended)
    Rs256,

    /// RSASSA-PKCS1-v1_5 using SHA-384 (Optional)
    Rs384,

    /// RSASSA-PKCS1-v1_5 using SHA-512 (Optional)
    Rs512,

    /// No digital signature or MAC performed (Optional)
    ///
    /// This variant is renamed as `Null` to avoid colliding with `Option::None`.
    #[serde(rename = "none")]
    Null,
}

impl fmt::Display for Signing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::prelude::rust_2021::*;
    use std::vec;

    use super::*;

    #[test]
    fn signing_algs() {
        use Signing::*;

        let input = vec![
            EdDsa, Es256, Es256K, Es384, Es512, Hs256, Hs384, Hs512, Ps256, Ps384, Ps512, Rs256, Rs384, Rs512, Null,
        ];
        let ser = serde_json::to_string(&input).expect("serialization failed");

        assert_eq!(
            ser,
            r#"["EdDSA","ES256","ES256K","ES384","ES512","HS256","HS384","HS512","PS256","PS384","PS512","RS256","RS384","RS512","none"]"#
        );

        assert_eq!(serde_json::from_str::<Vec<Signing>>(&ser).expect("deserialization failed"), input);
    }
}
