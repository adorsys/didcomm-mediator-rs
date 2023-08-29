use super::traits::{CoreSign, Error, Generate, KeyMaterial};
use super::AsymmetricKey;
use arrayref::array_ref;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH};

pub type Ed25519KeyPair = AsymmetricKey<VerifyingKey, SigningKey>;

impl std::fmt::Debug for Ed25519KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.public_key))
    }
}

impl KeyMaterial for Ed25519KeyPair {
    fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key.as_bytes().to_vec()
    }

    fn private_key_bytes(&self) -> Vec<u8> {
        self.secret_key.as_ref().map_or(vec![], |x| x.to_bytes().to_vec())
    }
}

impl Generate for Ed25519KeyPair {
    fn new() -> Ed25519KeyPair {
        Self::new_with_seed(vec![].as_slice())
    }

    fn new_with_seed(seed: &[u8]) -> Ed25519KeyPair {
        let secret_seed = crate::crypto::utils::generate_seed(seed).expect("invalid seed");

        let sk: SigningKey = SigningKey::from_bytes(&secret_seed); //.expect("cannot generate secret key");
        let pk: VerifyingKey = (&sk).try_into().expect("cannot generate public key");

        Ed25519KeyPair {
            secret_key: Some(sk),
            public_key: pk,
        }
    }

    fn from_public_key(public_key: &[u8]) -> Ed25519KeyPair {
        let public_key_fix: &[u8; PUBLIC_KEY_LENGTH] = array_ref!(public_key, 0, 32);
        Ed25519KeyPair {
            public_key: VerifyingKey::from_bytes(public_key_fix).expect("invalid byte data"),
            secret_key: None,
        }
    }

    fn from_secret_key(secret_key: &[u8]) -> Ed25519KeyPair {
        let secret_key_fix: &[u8; PUBLIC_KEY_LENGTH] = array_ref!(secret_key, 0, 32);
        let sk: SigningKey = SigningKey::from_bytes(secret_key_fix);
        let pk: VerifyingKey = (&sk).try_into().expect("cannot generate public key");

        Ed25519KeyPair {
            secret_key: Some(sk),
            public_key: pk,
        }
    }
}

impl CoreSign for Ed25519KeyPair {
    fn sign(&self, payload: &[u8]) -> Vec<u8> {
        if let Some(secret_key) = &self.secret_key {
            match secret_key.try_sign(payload) {
                Ok(signature) => signature.to_bytes().to_vec(),
                Err(_) => {
                    // Handle the error case here
                    Vec::new() // Return an empty vector as a default value
                }
            }
        } else {
            Vec::new() // Return an empty vector if there's no secret key
        }
    }

    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error> {
        let sig = Signature::try_from(signature).unwrap();
        match self.public_key.verify(payload, &sig) {
            Ok(_) => Ok(()),
            _ => Err(Error::Unknown("verify failed".into())),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use ed25519_dalek::{Signature, Verifier};

    use crate::crypto::traits::{Generate, KeyMaterial, CoreSign};

    use super::Ed25519KeyPair;

    // A test to create a new Ed25519KeyPair and check that bytes of both private and public key from
    // key material is 32 bytes long.
    #[test]
    fn test_new() {
        let keypair = Ed25519KeyPair::new();
        assert_eq!(keypair.public_key_bytes().len(), 32);
        assert_eq!(keypair.private_key_bytes().len(), 32);
    }

    // Generate a new Ed25519KeyPair with a seed and check that bytes of both private and public key
    // are equals to the given bytes pub_key_hex and pri_key_hex.
    #[test]
    fn test_new_with_seed() {
        // generate seed bytes from the the string "Sample seed bytes of thirtytwo!b"
        // Beware that you need a seed of 32 bytes to produce the deterministic key pair.
        let my_string = String::from("Sample seed bytes of thirtytwo!b");
        let seed: &[u8] = my_string.as_bytes();
        let keypair = Ed25519KeyPair::new_with_seed(seed);
        let pub_key_hex = hex::encode(keypair.public_key_bytes());
        let pri_key_hex = hex::encode(keypair.private_key_bytes());
        assert_eq!(pub_key_hex, "412328b0201b71d0144a27d028057b6fdf58d22e0f3baaebaa5388140e57bbbd");
        assert_eq!(pri_key_hex, "53616d706c652073656564206279746573206f662074686972747974776f2162");
    }

    // Creat a test that:
    // - Generate a key pair
    // - load the file test_resources/didcore_example_01.json
    // - sign the content of the file wiht the key pair
    // - Verify the signature 
    #[test]
    fn test_sign_verify() {
        let keypair = Ed25519KeyPair::new();
        let verifying_key = keypair.public_key;

        let json_file = "test_resources/crypto_ed25519_test_sign_verify.json";
        let json_data = std::fs::read_to_string(json_file).unwrap();

        let signature = keypair.sign(json_data.as_bytes());

        // Verify the signature
        let sig = Signature::try_from(&signature[..]).unwrap();
        let verified = verifying_key.verify(json_data.as_bytes(), &sig);
        assert!(verified.is_ok());
    }

}
