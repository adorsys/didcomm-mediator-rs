#[allow(unused)]
const DIDDOC_DIR: &str = "storage";
const KEYSTORE_DIR: &str = "storage/keystore";

/// Program entry
fn main() -> std::io::Result<()> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable tracing
    tracing_subscriber::fmt::init();

    // Read secret for key encryption
    let secret = std::env::var("DIDGEN_SECRET").expect("Please provide a secret key.");

    // Init store with timestamp-aware path
    let path = format!("{KEYSTORE_DIR}/{}.yaml", chrono::Utc::now().timestamp());
    let store = keystore::init_store(&path);
    tracing::info!("keystore: {path}");

    // Generate authentication key
    tracing::info!("Generating authentication key...");
    let authentication_key = keystore::gen_signing_keys(&store, &secret);

    // Generate assertion key
    tracing::info!("Generating assertion key...");
    let assertion_key = keystore::gen_signing_keys(&store, &secret);

    // Build DID document
    gen_diddoc(&authentication_key, &assertion_key);

    tracing::info!("Successful completion.");
    Ok(())
}

/// Builds and persists DID document
fn gen_diddoc(authentication_key: &String, assertion_key: &String) {
    println!("authentication: {authentication_key}");
    println!("assertion: {assertion_key}");
}

mod keystore {
    use multibase::Base::Base58Btc;
    use rand::rngs::OsRng;
    use rustbreak::{deser::Yaml, FileDatabase};
    use std::collections::HashMap;

    type FileKeyStore = FileDatabase<HashMap<String, String>, Yaml>;

    /// Initializes file-based key-value store.
    pub fn init_store(path: &str) -> FileKeyStore {
        FileKeyStore::create_at_path(path, HashMap::new()).unwrap()
    }

    /// Generates and persists ed25519 keys for digital signatures.
    /// Returns multibase-encoded public key for convenience.
    pub fn gen_signing_keys(store: &FileKeyStore, secret: &String) -> String {
        use ed25519_dalek::{
            pkcs8::{spki::der::pem::LineEnding, EncodePrivateKey},
            SigningKey,
        };

        // Generate
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);

        // Encode
        let prvkey = signing_key
            .to_pkcs8_encrypted_pem(OsRng, secret, LineEnding::LF)
            .unwrap()
            .to_string();
        let pubkey = multibase::encode(Base58Btc, signing_key.verifying_key().to_bytes());

        // Add to store
        store
            .write(|db| {
                db.insert(pubkey.clone(), prvkey);
            })
            .unwrap();

        // Persist
        store.save().expect("persist error");

        // Return public key
        pubkey
    }
}

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
