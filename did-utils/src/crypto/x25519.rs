use x25519_dalek::{PublicKey, StaticSecret};

use super::errors::Error;
use super::traits::BYTES_LENGTH_32;
use super::utils::{clone_slice_to_array, generate_seed};
use super::{
    traits::{Generate, KeyMaterial, ECDH},
    AsymmetricKey,
};

pub type X25519KeyPair = AsymmetricKey<PublicKey, StaticSecret>;

impl KeyMaterial for X25519KeyPair {
    fn public_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error> {
        Ok(clone_slice_to_array(self.public_key.as_bytes()))
    }

    fn private_key_bytes(&self) -> Result<[u8; BYTES_LENGTH_32], Error> {
        match &self.secret_key {
            Some(sk) => Ok(clone_slice_to_array(sk.as_bytes())),
            None => Err(Error::InvalidSecretKey),
        }
    }
}

impl Generate for X25519KeyPair {
    fn new() -> Result<X25519KeyPair, Error> {
        Self::new_with_seed(vec![].as_slice())
    }

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
    /// Performs an Elliptic Curve Diffie-Hellman (ECDH) key exchange.
    ///
    /// This method computes a shared secret using the given public key and the private key
    /// of this key pair. The shared secret can be used for cryptographic purposes such as
    /// deriving encryption keys.
    /// 
    /// # Example
    /// 
    /// ```
    /// use did_utils::crypto::{X25519KeyPair, ECDH};
    /// 
    /// let keypair = X25519KeyPair::new()?;
    /// let other_keypair = X25519KeyPair::new()?;
    /// let shared_secret = keypair.key_exchange(&other_keypair)?;
    /// let other_shared_secret = other_keypair.key_exchange(&keypair)?;
    /// assert_eq!(shared_secret, other_shared_secret);
    /// ```
    fn key_exchange(&self, key: &Self) -> Option<Vec<u8>> {
        (self.secret_key).as_ref().map(|x| x.diffie_hellman(&key.public_key).as_bytes().to_vec())
    }
}

#[cfg(test)]
pub mod tests {
    // use ed25519_dalek::{Signature, Verifier};

    use x25519_dalek::{EphemeralSecret, PublicKey};

    use super::X25519KeyPair;
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
}
