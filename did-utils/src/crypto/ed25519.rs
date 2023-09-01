use super::traits::{CoreSign, Error, Generate, KeyMaterial, BYTES_LENGTH_32};
use super::utils::{generate_seed, clone_slice_to_array};
use super::x25519::X25519KeyPair;
use super::AsymmetricKey;
use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha512};

pub type Ed25519KeyPair = AsymmetricKey<VerifyingKey, SigningKey>;

impl std::fmt::Debug for Ed25519KeyPair {
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
            Some(sk) => {
                Ok(clone_slice_to_array(&sk.to_bytes()))
            },
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
            BYTES_LENGTH_32 => {
                Ok(Ed25519KeyPair {
                    public_key: match VerifyingKey::from_bytes(&clone_slice_to_array(public_key)) {
                        Ok(vk) => vk,
                        Err(_) => return Err(Error::InvalidPublicKey),
                    },    
                    secret_key: None,
                })                
            }
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
    fn sign(&self, payload: &[u8]) -> Option<Vec<u8>> {
        match &self.secret_key {
            Some(sk) => {
                match sk.try_sign(payload) {
                    Ok(signature) => Some(signature.to_bytes().to_vec()),
                    Err(_) => None,
                }
            },
            None => None,
        }
    }

    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error> {
        match Signature::try_from(signature) {
            Ok(sig) => {
                match self.public_key.verify(payload, &sig) {
                    Ok(_) => Ok(()),
                    _ => Err(Error::VerificationError),
                }
            },
            Err(_) => Err(Error::CanNotRetrieveSignature),
        }
    }
}

impl Ed25519KeyPair {
    pub fn get_x25519(&self) -> Result<X25519KeyPair, Error> {
        match &self.secret_key {
            Some(sk) => {
                let bytes: [u8; BYTES_LENGTH_32] = sk.to_bytes();
                let mut hasher = Sha512::new();
                hasher.update(bytes);
                let hash = hasher.finalize();
                let mut output = [0u8; BYTES_LENGTH_32];
                output.copy_from_slice(&hash[..BYTES_LENGTH_32]);
                output[0] &= 248;
                output[31] &= 127;
                output[31] |= 64;

                X25519KeyPair::new_with_seed(&output)
            }
            None => {
                match self.public_key_bytes() {
                    Ok(pk_bytes) => {
                        match CompressedEdwardsY(pk_bytes).decompress() {
                            Some(point) => {
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
pub mod tests {
    use crate::crypto::traits::{CoreSign, Generate, KeyMaterial, BYTES_LENGTH_32};

    use super::Ed25519KeyPair;

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
}
