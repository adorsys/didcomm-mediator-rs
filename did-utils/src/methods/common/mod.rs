mod alg;
pub use alg::Algorithm;

use multibase::Base::Base58Btc;

use crate::crypto::{ed25519::Ed25519KeyPair, x25519::X25519KeyPair};

#[derive(Default)]
pub enum PublicKeyFormat {
    #[default]
    Multikey,
    Jwk,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_jwk::jwk::Jwk;

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
}
