use multibase::Base::Base58Btc;

use crate::{
    crypto::{
        ed25519::Ed25519KeyPair,
        traits::{Error as CryptoError, Generate, KeyMaterial},
    },
    methods::traits::DIDMethod,
};

#[allow(unused)]
#[non_exhaustive]
pub enum Algorithm {
    Ed25519,
    X25519,
}

impl Algorithm {
    pub fn muticodec_prefix(&self) -> [u8; 2] {
        match self {
            Self::Ed25519 => [0xed, 0x01],
            Self::X25519 => [0xec, 0x01],
        }
    }
}

#[derive(Default)]
pub enum PublicKeyFormat {
    #[default]
    Multikey,
    Jwk,
}

#[derive(Default)]
pub struct DIDKeyMethod {
    /// Key format to consider during DID
    /// expansion into a DID document.
    pub key_format: PublicKeyFormat,
}

impl DIDMethod for DIDKeyMethod {
    fn name() -> String {
        "did:key".to_string()
    }
}

impl DIDKeyMethod {
    /// Generates a did:key address ex-nihilo
    pub fn generate(&self) -> Result<String, CryptoError> {
        let keypair = Ed25519KeyPair::new()?;
        let multibase_value = multibase::encode(
            Base58Btc,
            [&Algorithm::Ed25519.muticodec_prefix(), keypair.public_key_bytes()?.as_slice()].concat(),
        );

        Ok(format!("did:key:{}", multibase_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_key_generation() {
        let did_method = DIDKeyMethod::default();
        let did = did_method.generate();

        assert!(did.is_ok());
        assert!(did.unwrap().starts_with("did:key:z6Mk"));
    }
}
