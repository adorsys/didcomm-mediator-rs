use did_utils::crypto::{
    traits::{Generate, KeyMaterial},
    x25519::X25519KeyPair,
};
use serde_json::Value;
use ssi::jwk::{Base64urlUInt, ECParams, OctetParams, Params, JWK};
use std::error::Error;

use crate::KEYSTORE_DIR;

pub struct KeyStore {
    path: String,
    keys: Vec<JWK>,
}

impl KeyStore {
    /// Constructs file-based key-value store.
    pub fn new() -> Self {
        Self {
            path: format!("{KEYSTORE_DIR}/{}.json", chrono::Utc::now().timestamp()),
            keys: vec![],
        }
    }

    /// Returns latest store on disk, if any.
    pub fn latest() -> Option<Self> {
        let msg = "Error parsing keystore directory";
        let file = std::fs::read_dir(KEYSTORE_DIR)
            .expect(msg)
            .map(|x| x.expect(msg).path().to_str().expect(msg).to_string())
            .filter(|p| p.ends_with(".json"))
            .max_by_key(|p| {
                let p = p
                    .trim_start_matches(&format!("{KEYSTORE_DIR}/"))
                    .trim_end_matches(".json");
                p.parse::<i32>().expect(msg)
            });

        match file {
            None => None,
            Some(path) => match std::fs::read_to_string(&path) {
                Err(_) => None,
                Ok(content) => match serde_json::from_str::<Vec<JWK>>(&content) {
                    Err(_) => None,
                    Ok(keys) => Some(Self { path, keys }),
                },
            },
        }
    }

    /// Gets path
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// Persists store on disk
    fn persist(&self) -> std::io::Result<()> {
        std::fs::write(self.path.clone(), serde_json::to_string_pretty(&self.keys)?)
    }

    /// Searches keypair given public key
    pub fn find_keypair(&self, pubkey: &JWK) -> Option<JWK> {
        self.keys.iter().find(|k| &k.to_public() == pubkey).cloned()
    }

    /// Generates and persists an ed25519 keypair for digital signatures.
    /// Returns public JWK for convenience.
    pub fn gen_ed25519_jwk(&mut self) -> Result<JWK, Box<dyn Error>> {
        let jwk = JWK::generate_ed25519()?;
        let pub_jwk = jwk.to_public();

        self.keys.push(jwk);
        self.persist()?;

        Ok(pub_jwk)
    }

    /// Generates and persists an x25519 keypair for digital signatures.
    /// Returns public JWK for convenience.
    pub fn gen_x25519_jwk(&mut self) -> Result<JWK, Box<dyn Error>> {
        let keypair = X25519KeyPair::new().map_err(|_| "Failure to generate X25519 keypair")?;
        let jwk = JWK::from(Params::OKP(OctetParams {
            curve: "X25519".to_string(),
            public_key: Base64urlUInt(keypair.public_key_bytes().unwrap().to_vec()),
            private_key: Some(Base64urlUInt(keypair.private_key_bytes().unwrap().to_vec())),
        }));
        let pub_jwk = jwk.to_public();

        self.keys.push(jwk);
        self.persist()?;

        Ok(pub_jwk)
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

    impl KeyStore {
        fn destroy(self) {
            std::fs::remove_file(self.path);
        }
    }

    #[test]
    fn test_keystore_flow() {
        let mut store = KeyStore::new();

        let jwk = store.gen_ed25519_jwk().unwrap();
        assert!(store.find_keypair(&jwk).is_some());

        let jwk = store.gen_x25519_jwk().unwrap();
        assert!(store.find_keypair(&jwk).is_some());

        let latest = KeyStore::latest();
        assert!(latest.is_some());

        store.destroy();
    }
}
