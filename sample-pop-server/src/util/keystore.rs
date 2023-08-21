use multibase::Base::Base58Btc;
use rand::rngs::OsRng;
use rustbreak::{deser::Yaml, FileDatabase};
use std::collections::HashMap;

use crate::KEYSTORE_DIR;

type FileKeyStore = FileDatabase<HashMap<String, String>, Yaml>;
pub struct KeyStore {
    path: String,
    store: FileKeyStore,
}

impl KeyStore {
    /// Constructs file-based key-value store.
    pub fn new() -> Self {
        let path = format!("{KEYSTORE_DIR}/{}.yaml", chrono::Utc::now().timestamp());

        Self {
            path: path.clone(),
            store: FileKeyStore::create_at_path(path, HashMap::new()).unwrap(),
        }
    }

    /// Returns internal store's path
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// Generates and persists ed25519 keys for digital signatures.
    /// Returns multibase-encoded public key for convenience.
    pub fn gen_signing_keys(&self, _secret: &str) -> String {
        use ed25519_dalek::{
            pkcs8::{spki::der::pem::LineEnding, EncodePrivateKey},
            SigningKey,
        };

        // Generate
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);

        // Encode
        let prvkey = signing_key
            // .to_pkcs8_encrypted_pem(OsRng, _secret, LineEnding::LF)
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap()
            .to_string();
        let pubkey = multibase::encode(Base58Btc, signing_key.verifying_key().to_bytes());

        // Add to store
        self.store
            .write(|db| {
                db.insert(pubkey.clone(), prvkey);
            })
            .unwrap();

        // Persist
        self.store.save().expect("persist error");

        // Return public key
        pubkey
    }
}

impl Default for KeyStore {
    fn default() -> Self {
        Self::new()
    }
}
