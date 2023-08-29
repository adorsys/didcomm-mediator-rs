use x25519_dalek::{PublicKey, StaticSecret};

use super::utils::{copy_slice_to_array, generate_seed, BYTES_LENGTH_32};
use super::{
    ed25519::Ed25519KeyPair,
    traits::{Generate, KeyMaterial, ECDH},
    AsymmetricKey,
};

pub type X25519KeyPair = AsymmetricKey<PublicKey, StaticSecret>;

impl std::fmt::Debug for X25519KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.public_key))
    }
}

impl KeyMaterial for X25519KeyPair {
    fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key.to_bytes().to_vec()
    }

    fn private_key_bytes(&self) -> Vec<u8> {
        self.secret_key.as_ref().unwrap().to_bytes().to_vec()
    }
}

impl Generate for X25519KeyPair {
    fn new() -> Self {
        Self::new_with_seed(vec![].as_slice())
    }

    fn new_with_seed(seed: &[u8]) -> Self {
        let secret_seed = generate_seed(seed).expect("invalid seed");

        let sk = StaticSecret::from(secret_seed);
        let pk: PublicKey = (&sk).try_into().expect("invalid public key");

        X25519KeyPair {
            public_key: pk,
            secret_key: Some(sk),
        }
    }

    fn from_public_key(public_key: &[u8]) -> Self {
        let mut pk: [u8; BYTES_LENGTH_32] = [0; BYTES_LENGTH_32];
        pk.clone_from_slice(public_key);

        X25519KeyPair {
            public_key: PublicKey::from(pk),
            secret_key: None,
        }
    }

    fn from_secret_key(secret_key: &[u8]) -> Self {
        let sized_data: [u8; BYTES_LENGTH_32] = copy_slice_to_array(&secret_key[..BYTES_LENGTH_32]).expect("Invalid byte length");

        let sk = StaticSecret::from(sized_data);
        let pk: PublicKey = (&sk).try_into().expect("invalid public key");

        X25519KeyPair {
            public_key: pk,
            secret_key: Some(sk),
        }
    }
}

impl ECDH for X25519KeyPair {
    fn key_exchange(&self, key: &Self) -> Vec<u8> {
        match &(self.secret_key) {
            Some(x) => x.diffie_hellman(&key.public_key).as_bytes().to_vec(),
            None => panic!("secret key not present"),
        }
    }
}

impl From<Ed25519KeyPair> for X25519KeyPair {
    fn from(key: Ed25519KeyPair) -> Self {
        key.get_x25519()
    }
}

#[cfg(test)]
pub mod tests {
    // use ed25519_dalek::{Signature, Verifier};

    use x25519_dalek::{EphemeralSecret, PublicKey};

    use super::X25519KeyPair;
    use crate::crypto::traits::{Generate, KeyMaterial, ECDH};

    // A test to create a new X25519KeyPair and check that bytes of both private and public key from
    // key material is 32 bytes long.
    #[test]
    fn test_new() {
        let keypair = X25519KeyPair::new();
        assert_eq!(keypair.public_key_bytes().len(), 32);
        assert_eq!(keypair.private_key_bytes().len(), 32);
    }

    // Generate a new X25519KeyPair with a seed and check that bytes of both private and public key
    // are equals to the given bytes pub_key_hex and pri_key_hex.
    #[test]
    fn test_new_with_seed() {
        // generate seed bytes from the the string "Sample seed bytes of thirtytwo!b"
        // Beware that you need a seed of 32 bytes to produce the deterministic key pair.
        let my_string = String::from("Sample seed bytes of thirtytwo!b");
        let seed: &[u8] = my_string.as_bytes();
        let keypair = X25519KeyPair::new_with_seed(seed);
        let pub_key_hex = hex::encode(keypair.public_key_bytes());
        let pri_key_hex = hex::encode(keypair.private_key_bytes());

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
        let decryption_keypair_at_recipient = X25519KeyPair::new();
        let encryption_public_key_bytes_at_recipient = decryption_keypair_at_recipient.public_key_bytes();

        // === Sender ===
        let encryption_public_key_bytes_at_sender = encryption_public_key_bytes_at_recipient.to_vec();
        // Generate an ephemeral secret keypair for the sender
        let eph_secret_at_sender = EphemeralSecret::random();
        // Store the ephemeral public key for transport in the header of message
        let eph_public_key_in_transport = PublicKey::from(&eph_secret_at_sender);
        // generate shared secret for use in symmetric encryption
        let encryption_public_key_at_sender = X25519KeyPair::from_public_key(&encryption_public_key_bytes_at_sender);
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

        let eph_public_key_at_recipient = X25519KeyPair::from_public_key(eph_public_key_in_transport.as_bytes());
        decryption_keypair_at_recipient.key_exchange(&eph_public_key_at_recipient);
        let symmetric_key_at_recipient = shared_secret_at_recipient.as_bytes();

        // Both equals assumes that encrypted payload will be successfuly decrypted.
        assert_eq!(symmetric_key_at_sender, symmetric_key_at_recipient);
    }
}
