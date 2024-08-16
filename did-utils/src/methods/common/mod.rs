mod alg;
pub use alg::Algorithm;

use multibase::Base::Base58Btc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::crypto::{Ed25519KeyPair, X25519KeyPair};

#[derive(Default)]
pub enum PublicKeyFormat {
    #[default]
    Multikey,
    Jwk,
}

#[allow(unused)]
pub trait ToMultikey {
    /// Converts keypair into its multikey string
    fn to_multikey(&self) -> String;
}

impl ToMultikey for Ed25519KeyPair {
    fn to_multikey(&self) -> String {
        let prefix = &Algorithm::Ed25519.muticodec_prefix();
        let bytes = &self.public_key.as_bytes()[..];
        multibase::encode(Base58Btc, [prefix, bytes].concat())
    }
}

impl ToMultikey for X25519KeyPair {
    fn to_multikey(&self) -> String {
        let prefix = &Algorithm::X25519.muticodec_prefix();
        let bytes = &self.public_key.as_bytes()[..];
        multibase::encode(Base58Btc, [prefix, bytes].concat())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Error)]
pub(super) enum DecodeMultikeyError {
    #[error("error to multibase decode")]
    MultibaseDecodeError,
    #[error("not multibase-encoded in Base58")]
    NotBase58MultibaseEncoded,
    #[error("assumed multicodec too short")]
    MulticodecTooShort,
    #[error("unknown algorithm")]
    UnknownAlgorithm,
}

/// Decodes algorithm and key bytes from multibase-encode value
pub(super) fn decode_multikey(multikey: &str) -> Result<(Algorithm, Vec<u8>), DecodeMultikeyError> {
    let (base, multicodec) = multibase::decode(multikey).map_err(|_| DecodeMultikeyError::MultibaseDecodeError)?;

    // Validate decoded multibase value: base
    if base != Base58Btc {
        return Err(DecodeMultikeyError::NotBase58MultibaseEncoded);
    }

    // Validate decoded multibase value: multicodec
    if multicodec.len() < 2 {
        return Err(DecodeMultikeyError::MulticodecTooShort);
    }

    // Partition multicodec value
    let multicodec_prefix: &[u8; 2] = &multicodec[..2].try_into().unwrap();
    let raw_public_key_bytes = &multicodec[2..];

    // Derive algorithm from multicodec prefix
    let alg = Algorithm::from_muticodec_prefix(multicodec_prefix).ok_or(DecodeMultikeyError::UnknownAlgorithm)?;

    // Output
    Ok((alg, raw_public_key_bytes.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_jwk::Jwk;

    use multibase::Base::Base64Url;

    #[test]
    fn test_ed25519_keypair_to_multikey() {
        let jwk: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "O2onvM62pC1io6jQKm8Nc2UyFXcd4kOmOsBIoYtZ2ik"
            }"#,
        )
        .unwrap();

        let keypair: Ed25519KeyPair = jwk.try_into().unwrap();
        let multikey = keypair.to_multikey();

        assert_eq!(&multikey, "z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp");
    }

    #[test]
    fn test_x25519_keypair_to_multikey() {
        let jwk: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU"
            }"#,
        )
        .unwrap();

        let keypair: X25519KeyPair = jwk.try_into().unwrap();
        let multikey = keypair.to_multikey();

        assert_eq!(&multikey, "z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr");
    }

    #[test]
    fn test_decode_multikey() {
        let multikey = "z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp";
        let (alg, bytes) = decode_multikey(multikey).unwrap();
        assert_eq!(alg, Algorithm::Ed25519);
        assert_eq!(bytes, Base64Url.decode("O2onvM62pC1io6jQKm8Nc2UyFXcd4kOmOsBIoYtZ2ik").unwrap());

        let multikey = "z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr";
        let (alg, bytes) = decode_multikey(multikey).unwrap();
        assert_eq!(alg, Algorithm::X25519);
        assert_eq!(bytes, Base64Url.decode("A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU").unwrap());
    }

    #[test]
    fn test_decode_multikey_negative_cases() {
        let cases = [
            (
                "z#6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWpd", 
                DecodeMultikeyError::MultibaseDecodeError,
            ),
            (
                "Z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK", //
                DecodeMultikeyError::NotBase58MultibaseEncoded,
            ),
            (
                "z6", //
                DecodeMultikeyError::MulticodecTooShort,
            ),
            (
                "z7MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWpd", //
                DecodeMultikeyError::UnknownAlgorithm,
            ),
        ];

        for (multikey, expected_err) in cases {
            let err = decode_multikey(multikey).unwrap_err();
            assert_eq!(err, expected_err);
        }
    }
}
