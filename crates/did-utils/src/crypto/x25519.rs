use multibase::Base::Base58Btc;
use x25519_dalek::{PublicKey, StaticSecret};

use super::{
    alg::Algorithm,
    errors::Error,
    traits::{Generate, KeyMaterial, ToMultikey, BYTES_LENGTH_32, ECDH},
    utils::{clone_slice_to_array, generate_seed},
    AsymmetricKey,
};

pub type X25519KeyPair = AsymmetricKey<PublicKey, StaticSecret>;

impl std::fmt::Debug for X25519KeyPair {
    /// Returns a string representation of the public key.
    ///
    /// This function is used to implement the `fmt::Debug` trait.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.public_key))
    }
}

impl KeyMaterial for X25519KeyPair {
    /// Returns the bytes of the public key.
    ///
    /// # Returns
    ///
    /// A `Result` containing the bytes of the public key or an `Error`.
    fn public_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error> {
        Ok(clone_slice_to_array(self.public_key.as_bytes()))
    }

    /// Returns the bytes of the private key.
    ///
    /// # Returns
    ///
    /// A `Result` containing the bytes of the private key or an `Error`.
    fn private_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error> {
        match &self.secret_key {
            Some(sk) => Ok(clone_slice_to_array(sk.as_bytes())),
            None => Err(Error::InvalidSecretKey),
        }
    }
}

impl Generate for X25519KeyPair {
    /// Generates a new X25519 key pair.
    ///
    /// If the initial seed is empty or invalid, a random seed will be generated.
    ///
    /// # Arguments
    ///
    /// * `seed` - The initial seed to use, or empty if none.
    ///
    /// # Returns
    ///
    /// A new `X25519KeyPair` instance.
    fn new() -> Result<X25519KeyPair, Error> {
        Self::new_with_seed(vec![].as_slice())
    }

    /// Generates a new X25519 key pair with a seed.
    ///
    /// If the seed is empty or invalid, generates a new seed.
    ///
    /// # Arguments
    ///
    /// * `seed` - The initial seed to use.
    ///
    /// # Returns
    ///
    /// A new `X25519KeyPair` instance.
    fn new_with_seed(seed: &[u8]) -> Result<X25519KeyPair, Error> {
        match generate_seed(seed) {
            Ok(secret_seed) => {
                let sk = StaticSecret::from(secret_seed);
                Ok(X25519KeyPair {
                    public_key: PublicKey::from(&sk),
                    secret_key: Some(sk),
                })
            }
            Err(_) => Err(Error::InvalidSeed),
        }
    }

    /// Creates a new `X25519KeyPair` from a public key.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The bytes of the public key.
    ///
    /// # Returns
    ///
    /// A new `X25519KeyPair` instance.
    fn from_public_key(public_key: &[u8; BYTES_LENGTH_32]) -> Result<X25519KeyPair, Error> {
        match public_key.len() {
            BYTES_LENGTH_32 => {
                let pk = clone_slice_to_array(public_key);
                Ok(X25519KeyPair {
                    public_key: PublicKey::from(pk),
                    secret_key: None,
                })
            }
            _ => Err(Error::InvalidKeyLength),
        }
    }

    /// Creates a new `X25519KeyPair` from a secret key.
    ///
    /// # Arguments
    ///
    /// * `secret_key` - The bytes of the secret key.
    ///
    /// # Returns
    ///
    /// A new `X25519KeyPair` instance.
    fn from_secret_key(secret_key: &[u8; BYTES_LENGTH_32]) -> Result<X25519KeyPair, Error> {
        match secret_key.len() {
            BYTES_LENGTH_32 => {
                let sk_bytes = clone_slice_to_array(secret_key);
                let sk = StaticSecret::from(sk_bytes);
                Ok(X25519KeyPair {
                    public_key: PublicKey::from(&sk),
                    secret_key: Some(sk),
                })
            }
            _ => Err(Error::InvalidKeyLength),
        }
    }
}

impl ECDH for X25519KeyPair {
    /// Performs a key exchange using the Diffie-Hellman algorithm.
    ///
    /// # Arguments
    ///
    /// * `key` - The public key of the other party.
    ///
    /// # Returns
    ///
    /// An optional vector of bytes representing the shared secret.
    /// If the secret key is not available, returns `None`.
    fn key_exchange(&self, key: &Self) -> Option<Vec<u8>> {
        (self.secret_key).as_ref().map(|x| x.diffie_hellman(&key.public_key).as_bytes().to_vec())
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
pub mod tests {
    // use ed25519_dalek::{Signature, Verifier};

    use crate::key_jwk::Jwk;
    use x25519_dalek::{EphemeralSecret, PublicKey};

    use super::*;
    use crate::crypto::{
        traits::{Generate, KeyMaterial, BYTES_LENGTH_32, ECDH},
        utils::clone_slice_to_array,
    };

    // A test to create a new X25519KeyPair and check that bytes of both private and public key from
    // key material is 32 bytes long.
    #[test]
    fn test_new() {
        let keypair = X25519KeyPair::new().unwrap();
        assert_eq!(keypair.public_key_bytes().unwrap().len(), BYTES_LENGTH_32);
        assert_eq!(keypair.private_key_bytes().unwrap().len(), BYTES_LENGTH_32);
    }

    // Generate a new X25519KeyPair with a seed and check that bytes of both private and public key
    // are equals to the given bytes pub_key_hex and pri_key_hex.
    #[test]
    fn test_new_with_seed() {
        // generate seed bytes from the the string "Sample seed bytes of thirtytwo!b"
        // Beware that you need a seed of 32 bytes to produce the deterministic key pair.
        let my_string = String::from("Sample seed bytes of thirtytwo!b");
        let seed: &[u8] = my_string.as_bytes();
        let keypair = X25519KeyPair::new_with_seed(seed).unwrap();
        let pub_key_hex = hex::encode(keypair.public_key_bytes().unwrap());
        let pri_key_hex = hex::encode(keypair.private_key_bytes().unwrap());

        assert_eq!(pub_key_hex, "2879534e09045c99580051db0cc7c0eac622a649b55893798fb62159f4134159");
        assert_eq!(pri_key_hex, "53616d706c652073656564206279746573206f662074686972747974776f2162");
    }

    // Creat a test that:
    // - Generate a key pair at the recipient side
    // - Encrypt the content of the file wiht the public key
    // - Use the secret key to decrypt the encrypted content.
    #[test]
    fn test_encrypt_decrypt() {
        // === Recipient ===
        // Generate a static secret keypair for the recipient
        let decryption_keypair_at_recipient = X25519KeyPair::new().unwrap();
        let encryption_public_key_bytes_at_recipient = decryption_keypair_at_recipient.public_key_bytes().unwrap();

        // === Sender ===
        let encryption_public_key_bytes_at_sender = clone_slice_to_array(&encryption_public_key_bytes_at_recipient);
        // Generate an ephemeral secret keypair for the sender
        let eph_secret_at_sender = EphemeralSecret::random();
        // Store the ephemeral public key for transport in the header of message
        let eph_public_key_in_transport = PublicKey::from(&eph_secret_at_sender);
        // generate shared secret for use in symmetric encryption
        let encryption_public_key_at_sender = X25519KeyPair::from_public_key(&encryption_public_key_bytes_at_sender).unwrap();
        let shared_secret_at_sender = &eph_secret_at_sender.diffie_hellman(&encryption_public_key_at_sender.public_key);
        // produce the key for use in symmetric encryption
        let symmetric_key_at_sender = shared_secret_at_sender.as_bytes();

        // === In Transport ===
        // Cyper message encrypted with the symmetric key
        // eph_public_key_in_transport

        // === Recipient ===
        // Construct EphPublicKey from message header
        // let eph_secret_at_recipient = EphemeralSecret::from(&eph_public_key);
        let shared_secret_at_recipient = decryption_keypair_at_recipient
            .secret_key
            .as_ref()
            .unwrap()
            .diffie_hellman(&eph_public_key_in_transport);

        let eph_public_key_at_recipient = X25519KeyPair::from_public_key(eph_public_key_in_transport.as_bytes()).unwrap();
        decryption_keypair_at_recipient.key_exchange(&eph_public_key_at_recipient);
        let symmetric_key_at_recipient = shared_secret_at_recipient.as_bytes();

        // Both equals assumes that encrypted payload will be successfuly decrypted.
        assert_eq!(symmetric_key_at_sender, symmetric_key_at_recipient);
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
