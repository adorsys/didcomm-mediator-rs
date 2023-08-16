const KEYSTORE_PATHSTR: &str = "storage/keystore";

fn main() {}

#[cfg(test)]
mod tests {
    #[test]
    fn can_jcs_serialize() {
        let data = serde_json::json!({
            "from_account": "543 232 625-3",
            "to_account": "321 567 636-4",
            "amount": 500.50,
            "currency": "USD"
        });

        let jcs = r#"{"amount":500.5,"currency":"USD","from_account":"543 232 625-3","to_account":"321 567 636-4"}"#;

        assert_eq!(jcs, json_canon::to_string(&data).unwrap());
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use std::path::Path;

    use signatory::{
        ecdsa::secp256k1::{Signature, SigningKey},
        signature::{Signer, Verifier},
        FsKeyStore, GeneratePkcs8, KeyName, KeyRing,
    };

    /// Integration test for loading a key from a keystore
    #[test]
    fn integration() {
        // let dir = tempfile::tempdir().unwrap();
        let dir = Path::new(KEYSTORE_PATHSTR);
        let key_store = FsKeyStore::create_or_open(&dir).unwrap();
        let example_key = SigningKey::generate_pkcs8();

        let key_name = "example".parse::<KeyName>().unwrap();
        key_store.store(&key_name, &example_key).unwrap();

        let mut key_ring = KeyRing::new();
        let key_handle = key_store.import(&key_name, &mut key_ring).unwrap();

        let signing_key = key_ring.ecdsa.secp256k1.iter().next().unwrap();
        let verifying_key = key_handle.ecdsa_secp256k1().unwrap();
        assert_eq!(signing_key.verifying_key(), verifying_key);

        let example_message = "Hello, world!";
        let signature: Signature = signing_key.sign(example_message.as_bytes());
        assert!(verifying_key
            .verify(example_message.as_bytes(), &signature)
            .is_ok());
    }
}
