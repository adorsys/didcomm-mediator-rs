use multibase::Base::Base58Btc;
use pkcs8::DecodePrivateKey;
use rand::rngs::OsRng;
use rustbreak::{deser::Yaml, FileDatabase, RustbreakError};
use std::collections::HashMap;

use ed25519_dalek::{
    pkcs8::{spki::der::pem::LineEnding, EncodePrivateKey},
    SigningKey,
};

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

    /// Returns latest store on disk, if any.
    pub fn latest() -> Option<Self> {
        let file = std::fs::read_dir(KEYSTORE_DIR)
            .unwrap()
            .map(|x| x.unwrap().path())
            .filter(|p| p.to_str().unwrap().ends_with(".yaml"))
            .max();

        file.map(|path| Self {
            path: path.to_str().unwrap().to_owned(),
            store: FileKeyStore::load_from_path(path).unwrap(),
        })
    }

    /// Returns internal store's path
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// Reads private key given public key
    pub fn lookup_signing_key(&self, pubkey: &str, _secret: &str) -> Option<SigningKey> {
        let mut result = None;

        // Read encrypted signing key from store
        self.store
            .read(|db| {
                result = match db.get(pubkey) {
                    Some(privkey) => SigningKey::from_pkcs8_pem(privkey).ok(),
                    None => None,
                }
            })
            .unwrap();

        result
    }

    /// Generates and persists ed25519 keys for digital signatures.
    /// Returns multibase-encoded public key for convenience.
    pub fn gen_signing_keys(&self, _secret: &str) -> String {
        // Generate
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);

        // Encode
        let prvkey = signing_key
            // TODO!!! This is too slow to run. Should discuss how appropriate it is.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_latest() {
        let store = KeyStore::latest();
        assert!(store.is_some());
    }
}
