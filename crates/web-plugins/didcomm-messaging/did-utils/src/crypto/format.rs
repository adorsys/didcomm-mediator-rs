use crate::{
    crypto::{
        ed25519::Ed25519KeyPair,
        errors::Error as CryptoError,
        traits::{Generate, KeyMaterial, BYTES_LENGTH_32},
        x25519::X25519KeyPair,
    },
    jwk::{
        Bytes, Jwk, Key, Parameters, Secret, {Okp, OkpCurves},
    },
};

use multibase::Base::Base64Url;

#[derive(Default)]
pub enum PublicKeyFormat {
    #[default]
    Multikey,
    Jwk,
}

impl TryFrom<Ed25519KeyPair> for Jwk {
    type Error = CryptoError;

    // Converts an Ed25519KeyPair to a Jwk.
    fn try_from(keypair: Ed25519KeyPair) -> Result<Self, Self::Error> {
        Ok(Jwk {
            key: Key::Okp(Okp {
                crv: OkpCurves::Ed25519,
                x: Bytes::from(keypair.public_key_bytes()?.to_vec()),
                d: Some(Secret::from(keypair.private_key_bytes()?.to_vec())),
            }),
            prm: Parameters::default(),
        })
    }
}

/// Converts a `Jwk` to an `Ed25519KeyPair`.
impl TryFrom<Jwk> for Ed25519KeyPair {
    type Error = CryptoError;

    // Converts a Jwk to an Ed25519KeyPair.
    fn try_from(jwk: Jwk) -> Result<Self, Self::Error> {
        match jwk.key {
            Key::Okp(okp) => {
                if okp.crv != OkpCurves::Ed25519 {
                    return Err(CryptoError::InvalidCurve);
                }
                match okp.d {
                    Some(secret_key) => {
                        let secret = secret_key;

                        let secret_key_vec = secret.to_vec();

                        let bytes: [u8; 32] = secret_key_vec.try_into().map_err(|_| CryptoError::InvalidSecretKey)?;
                        Ed25519KeyPair::from_secret_key(&bytes)
                    }
                    None => {
                        let public_key = okp.x;
                        let public_key_vec = public_key.to_vec();
                        Ed25519KeyPair::from_public_key(&public_key_vec.try_into().map_err(|_| CryptoError::InvalidPublicKey)?)
                    }
                }
            }
            _ => Err(CryptoError::Unsupported),
        }
    }
}

impl TryFrom<X25519KeyPair> for Jwk {
    type Error = CryptoError;

    // Converts an `X25519KeyPair` to a `Jwk`.
    fn try_from(keypair: X25519KeyPair) -> Result<Self, Self::Error> {
        Ok(Jwk {
            key: Key::Okp(Okp {
                crv: OkpCurves::X25519,
                x: Bytes::from(keypair.public_key_bytes()?.to_vec()),
                d: Some(Secret::from(keypair.private_key_bytes()?.to_vec())),
            }),
            prm: Parameters::default(),
        })
    }
}

impl TryFrom<Jwk> for X25519KeyPair {
    type Error = CryptoError;

    // Converts a `Jwk` to an `X25519KeyPair`.
    fn try_from(jwk: Jwk) -> Result<Self, Self::Error> {
        match jwk.key {
            Key::Okp(okp) => {
                if okp.crv != OkpCurves::X25519 {
                    return Err(CryptoError::InvalidCurve);
                }
                match okp.d {
                    Some(secret_key) => {
                        let secret = secret_key;

                        let secret_key_vec = secret.to_vec();

                        let bytes: [u8; 32] = secret_key_vec.try_into().map_err(|_| CryptoError::InvalidSecretKey)?;
                        X25519KeyPair::from_secret_key(&bytes)
                    }
                    None => {
                        let public_key = okp.x;
                        let public_key_vec = public_key.to_vec();
                        X25519KeyPair::from_public_key(&public_key_vec.try_into().map_err(|_| CryptoError::InvalidPublicKey)?)
                    }
                }
            }
            _ => Err(CryptoError::Unsupported),
        }
    }
}

// Decodes a base64url encoded key string to bytes.
#[allow(dead_code)]
fn base64url_to_bytes(key: &str) -> Result<[u8; BYTES_LENGTH_32], ()> {
    let key: Vec<u8> = Base64Url.decode(key).map_err(|_| ())?;
    let key: [u8; BYTES_LENGTH_32] = key.try_into().map_err(|_| ())?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{CoreSign, Generate, ECDH};

    // Tests conversion between Ed25519KeyPair and Jwk
    #[test]
    fn test_conversion_ed25519_jwk() -> Result<(), CryptoError> {
        let seed = b"TMwLj2p2qhcuVhaFAj3QkkJGhK6pdyKx";
        let payload = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

        let keypair = Ed25519KeyPair::new_with_seed(seed)?;
        let signature = keypair.sign(payload).unwrap();

        let jwk: Jwk = keypair.try_into()?;
        let keypair: Ed25519KeyPair = jwk.try_into()?;
        assert!(keypair.verify(payload, &signature).is_ok());

        Ok(())
    }

    // Tests conversion from Jwk to Ed25519KeyPair with external signature
    #[test]
    fn test_conversion_ed25519_jwk_with_external_signature() -> Result<(), CryptoError> {
        let jwk: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "tjOTPcs4OEMNrmn2ScYZDS-aCCbRFhJgaAmGnRsdmEo"
            }"#,
        )
        .unwrap();

        let payload = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
        let signature = "2QH7Qrt8clEn4ETh9lgcGUyo26cJj1U8U0CBFQvgCWHe1dwXXXb16SzPTVNVGm-J6m6eALjWrxuJfmbApdoBAQ";
        let signature = Base64Url.decode(signature).unwrap();

        let keypair: Ed25519KeyPair = jwk.try_into()?;
        assert!(keypair.verify(payload, &signature).is_ok());

        Ok(())
    }

    // Tests conversion between X25519KeyPair and Jwk
    #[test]
    fn test_conversion_x25519_jwk() -> Result<(), CryptoError> {
        let alice_seed = b"TMwLj2p2qhcuVhaFAj3QkkJGhK6pdyKx";
        let bob_seed = b"NWB6DbnIlewWVp5jIJOSgyX8msXNPPAL";

        let alice = X25519KeyPair::new_with_seed(alice_seed)?;
        let bob = X25519KeyPair::new_with_seed(bob_seed)?;

        let alice_shared_secret = alice.key_exchange(&bob);

        let alice_jwk: Jwk = alice.try_into()?;
        let alice: X25519KeyPair = alice_jwk.try_into()?;
        let bob_jwk: Jwk = bob.try_into()?;
        let bob: X25519KeyPair = bob_jwk.try_into()?;

        let bob_shared_secret = bob.key_exchange(&alice);

        assert_eq!(alice_shared_secret, bob_shared_secret);
        Ok(())
    }
}
