extern crate did_utils;
use did_utils::crypto::{
    ed25519::Ed25519KeyPair,
    traits::{CoreSign, Generate},
};

fn main() {
    let keypair = Ed25519KeyPair::new().unwrap();

    let json_file = "test_resources/crypto_ed25519_test_sign_verify.json";
    let json_data = std::fs::read_to_string(json_file).unwrap();

    let signature = keypair.sign(json_data.as_bytes()).unwrap();

    // Verify the signature
    let verified = keypair.verify(json_data.as_bytes(), &signature);
    assert!(verified.is_ok());
}
