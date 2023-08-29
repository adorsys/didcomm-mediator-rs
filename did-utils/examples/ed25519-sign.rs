extern crate did_utils;
use did_utils::crypto::{ed25519::Ed25519KeyPair, traits::{Generate, CoreSign}};
use ed25519_dalek::{Signature, Verifier};


fn main() {
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
