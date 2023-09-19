use multibase::Base::Base58Btc;

use crate::{
    crypto::{
        ed25519::Ed25519KeyPair,
        traits::{Error as CryptoError, Generate, KeyMaterial},
    },
    methods::traits::DIDMethod,
};

use super::alg::Algorithm;

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
    /// Generates did:key address ex nihilo, off self-generated Ed25519 key pair
    pub fn generate() -> Result<String, CryptoError> {
        let keypair = Ed25519KeyPair::new()?;
        Self::from_ed25519_keypair(&keypair)
    }

    /// Computes did:key address corresponding to Ed25519 key pair
    pub fn from_ed25519_keypair(keypair: &Ed25519KeyPair) -> Result<String, CryptoError> {
        let multibase_value = multibase::encode(
            Base58Btc,
            [&Algorithm::Ed25519.muticodec_prefix(), keypair.public_key_bytes()?.as_slice()].concat(),
        );

        Ok(format!("did:key:{}", multibase_value))
    }

    /// Computes did:key address corresponding to raw public key bytes
    pub fn from_raw_public_key(alg: Algorithm, bytes: &[u8]) -> Result<String, CryptoError> {
        if let Some(required_length) = alg.public_key_length() {
            if required_length != bytes.len() {
                return Err(CryptoError::InvalidKeyLength);
            }
        }

        let multibase_value = multibase::encode(Base58Btc, [&alg.muticodec_prefix(), bytes].concat());

        Ok(format!("did:key:{}", multibase_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::didcore::Jwk;

    #[test]
    fn test_did_key_generation() {
        let did = DIDKeyMethod::generate();
        assert!(did.unwrap().starts_with("did:key:z6Mk"));
    }

    #[test]
    fn test_did_key_generation_from_given_jwk() {
        let jwk: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "O2onvM62pC1io6jQKm8Nc2UyFXcd4kOmOsBIoYtZ2ik"
            }"#,
        )
        .unwrap();
        let keypair: Ed25519KeyPair = jwk.try_into().unwrap();

        let did = DIDKeyMethod::from_ed25519_keypair(&keypair);
        assert_eq!(did.unwrap(), "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp");
    }

    #[test]
    fn test_did_key_generation_from_given_key_material() {
        let entries = [
            (
                Algorithm::Ed25519,
                hex::decode("3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29").unwrap(),
                "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            ),
            (
                Algorithm::X25519,
                hex::decode("2fe57da347cd62431528daac5fbb290730fff684afc4cfc2ed90995f58cb3b74").unwrap(),
                "did:key:z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F",
            ),
            (
                Algorithm::Secp256k1,
                hex::decode("03874c15c7fda20e539c6e5ba573c139884c351188799f5458b4b41f7924f235cd").unwrap(),
                "did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme",
            ),
        ];

        for entry in entries {
            let (alg, bytes, expected) = entry;
            let did = DIDKeyMethod::from_raw_public_key(alg, &bytes);
            assert_eq!(did.unwrap(), expected);
        }
    }
}
