use multibase::Base::Base58Btc;

use crate::{
    crypto::{
        ed25519::Ed25519KeyPair,
        traits::{Error as CryptoError, Generate, KeyMaterial},
    },
    methods::traits::DIDMethod,
};

#[non_exhaustive]
pub enum Algorithm {
    Ed25519,
    X25519,
}

impl Algorithm {
    pub fn muticodec_prefix(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 => vec![0xed],
            Self::X25519 => vec![0xec],
        }
    }
}

pub enum PublicKeyFormat {
    Multikey,
    Jwk,
}

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
    pub fn generate() -> Result<String, CryptoError> {
        let keypair = Ed25519KeyPair::new()?;
        let multibase_value = multibase::encode(
            Base58Btc,
            [Algorithm::Ed25519.muticodec_prefix().as_slice(), &keypair.public_key_bytes()?].concat(),
        );

        Ok(format!("did:key:{}", multibase_value))
    }
}
