use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use multibase::Base::Base58Btc;
use sha2::{Digest, Sha512};

use super::{
    alg::Algorithm,
    errors::Error,
    traits::{CoreSign, Generate, KeyMaterial, ToMultikey, BYTES_LENGTH_32},
    utils::{clone_slice_to_array, generate_seed},
    x25519::X25519KeyPair,
    AsymmetricKey,
};

/// A wrapper struct for an Ed25519 asymmetric key pair.
pub type Ed25519KeyPair = AsymmetricKey<VerifyingKey, SigningKey>;

impl std::fmt::Debug for Ed25519KeyPair {
    /// Returns a string representation of the public key.
    ///
    /// This function is used to implement the `fmt::Debug` trait.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.public_key))
    }
}

impl KeyMaterial for Ed25519KeyPair {
    fn public_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error> {
        Ok(clone_slice_to_array(self.public_key.as_bytes()))
    }

    fn private_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error> {
        match &self.secret_key {
            Some(sk) => Ok(clone_slice_to_array(&sk.to_bytes())),
            None => Err(Error::InvalidSecretKey),
        }
    }
}

impl Generate for Ed25519KeyPair {
    fn new() -> Result<Ed25519KeyPair, Error> {
        Self::new_with_seed(vec![].as_slice())
    }

    fn new_with_seed(seed: &[u8]) -> Result<Ed25519KeyPair, Error> {
        match generate_seed(seed) {
            Ok(secret_seed) => {
                let sk: SigningKey = SigningKey::from_bytes(&secret_seed);
                Ok(Ed25519KeyPair {
                    public_key: sk.verifying_key(),
                    secret_key: Some(sk),
                })
            }
            Err(_) => Err(Error::InvalidSeed),
        }
    }

    fn from_public_key(public_key: &[u8; BYTES_LENGTH_32]) -> Result<Ed25519KeyPair, Error> {
        match public_key.len() {
            BYTES_LENGTH_32 => Ok(Ed25519KeyPair {
                public_key: match VerifyingKey::from_bytes(&clone_slice_to_array(public_key)) {
                    Ok(vk) => vk,
                    Err(_) => return Err(Error::InvalidPublicKey),
                },
                secret_key: None,
            }),
            _ => Err(Error::InvalidKeyLength),
        }
    }

    fn from_secret_key(secret_key: &[u8; BYTES_LENGTH_32]) -> Result<Ed25519KeyPair, Error> {
        match secret_key.len() {
            BYTES_LENGTH_32 => {
                let sk: SigningKey = SigningKey::from_bytes(&clone_slice_to_array(secret_key));
                Ok(Ed25519KeyPair {
                    public_key: sk.verifying_key(),
                    secret_key: Some(sk),
                })
            }
            _ => Err(Error::InvalidKeyLength),
        }
    }
}

impl CoreSign for Ed25519KeyPair {
    /// Signs the given payload and returns the signature.
    ///
    /// The signature is generated using the private key of the `Ed25519KeyPair`.
    ///
    /// # Example
    ///
    /// ```
    /// use did_utils::crypto::{Ed25519KeyPair, CoreSign};
    ///
    /// let kp = Ed25519KeyPair::new()?;
    /// let signature = kp.sign(b"Hello, World!")?;
    /// ```
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Error> {
        // Check if the secret key is present
        match &self.secret_key {
            Some(sk) => {
                // Try to sign the payload
                match sk.try_sign(payload) {
                    Ok(signature) => {
                        // Convert the signature to bytes and return it
                        Ok(signature.to_bytes().to_vec())
                    }
                    Err(_) => Err(Error::SignatureError),
                }
            }
            None => Err(Error::InvalidSecretKey),
        }
    }

    /// Verifies the signature of the given payload.
    ///
    /// # Example
    ///
    /// ```
    /// use did_utils::crypto::{Ed25519KeyPair, CoreSign};
    ///
    /// let kp = Ed25519KeyPair::new()?;
    /// let signature = kp.sign(b"Hello, World!")?;
    /// let result = kp.verify(b"Hello, World!", &signature)?;
    ///
    /// assert!(result.is_ok());
    /// ```
    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error> {
        // Try to convert the signature to a `Signature` instance
        // This conversion is necessary because the `signature` argument is represented as bytes
        match Signature::try_from(signature) {
            Ok(sig) => match self.public_key.verify(payload, &sig) {
                Ok(_) => Ok(()),
                _ => Err(Error::VerificationError),
            },
            Err(_) => Err(Error::CanNotRetrieveSignature),
        }
    }
}

impl ToMultikey for Ed25519KeyPair {
    fn to_multikey(&self) -> String {
        let prefix = &Algorithm::Ed25519.muticodec_prefix();
        let bytes = &self.public_key.as_bytes()[..];
        multibase::encode(Base58Btc, [prefix, bytes].concat())
    }
}

impl Ed25519KeyPair {
    /// Converts an Ed25519 key pair to an X25519 key pair.
    ///
    /// If the secret key is present, it derives the X25519 key pair from the secret key.
    /// If the secret key is not present, it derives the X25519 key pair from the public key.
    ///
    /// # Example
    ///
    /// ```
    /// use did_utils::crypto::{Ed25519KeyPair, Generate, X25519KeyPair};
    ///
    /// let kp = Ed25519KeyPair::new()?;
    /// let xkp = kp.to_x25519()?;
    /// ```
    pub fn get_x25519(&self) -> Result<X25519KeyPair, Error> {
        // Check if the secret key is present
        match &self.secret_key {
            Some(sk) => {
                let bytes: [u8; BYTES_LENGTH_32] = sk.to_bytes();
                // Compute the SHA-512 hash of the secret key
                let mut hasher = Sha512::new();
                hasher.update(bytes);
                let hash = hasher.finalize();
                // Copy the first 32 bytes of the hash to the output buffer
                let mut output = [0u8; BYTES_LENGTH_32];
                output.copy_from_slice(&hash[..BYTES_LENGTH_32]);
                // Adjust the first byte and the last byte of the output buffer
                output[0] &= 248;
                output[31] &= 127;
                output[31] |= 64;

                // Create a new X25519 key pair using the output buffer
                X25519KeyPair::new_with_seed(&output)
            }
            None => {
                // Get the bytes of the public key
                match self.public_key_bytes() {
                    Ok(pk_bytes) => {
                        // Decompress the compressed Ed25519 point
                        match CompressedEdwardsY(pk_bytes).decompress() {
                            Some(point) => {
                                // Convert the point to Montgomery form and create a new X25519 key pair
                                let montgomery = point.to_montgomery();
                                X25519KeyPair::from_public_key(montgomery.as_bytes())
                            }
                            None => Err(Error::InvalidPublicKey),
                        }
                    }
                    Err(_) => Err(Error::InvalidPublicKey),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::traits::{CoreSign, Generate, KeyMaterial, BYTES_LENGTH_32};
    use crate::jwk::Jwk;

    // A test to create a new Ed25519KeyPair and check that bytes of both private and public key from
    // key material is 32 bytes long.
    #[test]
    fn test_new() {
        let keypair = Ed25519KeyPair::new().unwrap();
        assert_eq!(keypair.public_key_bytes().unwrap().len(), BYTES_LENGTH_32);
        assert_eq!(keypair.private_key_bytes().unwrap().len(), BYTES_LENGTH_32);
    }

    // Generate a new Ed25519KeyPair with a seed and check that bytes of both private and public key
    // are equals to the given bytes pub_key_hex and pri_key_hex.
    #[test]
    fn test_new_with_seed() {
        // generate seed bytes from the the string "Sample seed bytes of thirtytwo!b"
        // Beware that you need a seed of 32 bytes to produce the deterministic key pair.
        let my_string = String::from("Sample seed bytes of thirtytwo!b");
        let seed: &[u8] = my_string.as_bytes();
        let keypair = Ed25519KeyPair::new_with_seed(seed).unwrap();
        let pub_key_hex = hex::encode(keypair.public_key_bytes().unwrap());
        let pri_key_hex = hex::encode(keypair.private_key_bytes().unwrap());
        assert_eq!(pub_key_hex, "412328b0201b71d0144a27d028057b6fdf58d22e0f3baaebaa5388140e57bbbd");
        assert_eq!(pri_key_hex, "53616d706c652073656564206279746573206f662074686972747974776f2162");
    }

    // Creat a test that:
    // - Generate a key pair
    // - load the file test_resources/crypto_ed25519_test_sign_verify.json
    // - sign the content of the file wiht the key pair
    // - Verify the signature
    #[test]
    fn test_sign_verify() {
        let keypair = Ed25519KeyPair::new().unwrap();

        let json_file = "test_resources/crypto_ed25519_test_sign_verify.json";
        let json_data = std::fs::read_to_string(json_file).unwrap();

        let signature = keypair.sign(json_data.as_bytes());

        // Verify the signature
        let verified = keypair.verify(json_data.as_bytes(), &signature.unwrap());
        assert!(verified.is_ok());
    }

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
}
