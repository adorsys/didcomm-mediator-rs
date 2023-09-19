use multibase::Base::Base64Url;

use crate::didcore::Jwk;

#[derive(Copy, Clone)]
#[allow(unused, clippy::upper_case_acronyms)]
pub enum Algorithm {
    Ed25519,
    X25519,
    Secp256k1,
    BLS12381,
    P256,
    P384,
    P521,
    RSA,
}

use Algorithm::*;

impl Algorithm {
    pub fn muticodec_prefix(&self) -> [u8; 2] {
        match self {
            Ed25519 => [0xed, 0x01],
            X25519 => [0xec, 0x01],
            Secp256k1 => [0xe7, 0x01],
            BLS12381 => [0xeb, 0x01],
            P256 => [0x80, 0x24],
            P384 => [0x81, 0x24],
            P521 => [0x82, 0x24],
            RSA => [0x85, 0x24],
        }
    }

    pub fn from_muticodec_prefix(prefix: &[u8; 2]) -> Option<Self> {
        match prefix {
            [0xed, 0x01] => Some(Ed25519),
            [0xec, 0x01] => Some(X25519),
            [0xe7, 0x01] => Some(Secp256k1),
            [0xeb, 0x01] => Some(BLS12381),
            [0x80, 0x24] => Some(P256),
            [0x81, 0x24] => Some(P384),
            [0x82, 0x24] => Some(P521),
            [0x85, 0x24] => Some(RSA),
            _ => None,
        }
    }

    pub fn public_key_length(&self) -> Option<usize> {
        match self {
            Ed25519 => Some(32),
            X25519 => Some(32),
            Secp256k1 => Some(33),
            BLS12381 => Some(32),
            P256 => Some(33),
            P384 => Some(49),
            P521 => None,
            RSA => None,
        }
    }

    pub fn build_jwk(&self, raw_public_key_bytes: &[u8]) -> Jwk {
        use Algorithm::*;

        match self {
            Ed25519 => Jwk {
                key_id: None,
                key_type: String::from("OKP"),
                curve: String::from("Ed25519"),
                x: Some(Base64Url.encode(raw_public_key_bytes)),
                y: None,
                d: None,
            },
            X25519 => Jwk {
                key_id: None,
                key_type: String::from("OKP"),
                curve: String::from("X25519"),
                x: Some(Base64Url.encode(raw_public_key_bytes)),
                y: None,
                d: None,
            },
            // TODO! Extend implementation to other algorithms
            _ => unimplemented!(),
        }
    }
}
