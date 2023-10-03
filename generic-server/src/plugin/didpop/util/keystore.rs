use did_utils::{
    crypto::{ed25519::Ed25519KeyPair, traits::Generate, x25519::X25519KeyPair},
    didcore::Jwk,
};
use std::error::Error;

use crate::plugin::didpop::KEYSTORE_DIR;

pub struct KeyStore {
    path: String,
    keys: Vec<Jwk>,
}

struct KeyStoreFactory {
    location: String,
}

impl KeyStoreFactory {
    fn create(&self) -> KeyStore {
        KeyStore {
            path: format!("{}/{}.json", self.location, chrono::Utc::now().timestamp()),
            keys: vec![],
        }
    }

    fn latest(&self) -> Option<KeyStore> {
        let msg = "Error parsing keystore directory";
        let file = std::fs::read_dir(&self.location)
            .expect(msg)
            .map(|x| x.expect(msg).path().to_str().expect(msg).to_string())
            .filter(|p| p.ends_with(".json"))
            .max_by_key(|p| {
                let p = p
                    .trim_start_matches(&format!("{}/", self.location))
                    .trim_end_matches(".json");
                p.parse::<i32>().expect(msg)
            });

        match file {
            None => None,
            Some(path) => match std::fs::read_to_string(&path) {
                Err(_) => None,
                Ok(content) => match serde_json::from_str::<Vec<Jwk>>(&content) {
                    Err(_) => None,
                    Ok(keys) => Some(KeyStore { path, keys }),
                },
            },
        }
    }
}

impl KeyStore {
    /// Constructs file-based key-value store.
    pub fn new() -> Self {
        Self::factory(KEYSTORE_DIR).create()
    }

    /// Returns latest store on disk, if any.
    pub fn latest() -> Option<Self> {
        Self::factory(KEYSTORE_DIR).latest()
    }

    /// Returns location-aware factory
    fn factory(location: &str) -> KeyStoreFactory {
        KeyStoreFactory {
            location: location.to_string(),
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
    pub fn find_keypair(&self, pubkey: &Jwk) -> Option<Jwk> {
        self.keys.iter().find(|k| &k.to_public() == pubkey).cloned()
    }

    /// Generates and persists an ed25519 keypair for digital signatures.
    /// Returns public Jwk for convenience.
    pub fn gen_ed25519_jwk(&mut self) -> Result<Jwk, Box<dyn Error>> {
        let keypair = Ed25519KeyPair::new().map_err(|_| "Failure to generate Ed25519 keypair")?;
        let jwk: Jwk = keypair
            .try_into()
            .map_err(|_| "Failure to map to Jwk format")?;
        let pub_jwk = jwk.to_public();

        self.keys.push(jwk);
        self.persist()?;

        Ok(pub_jwk)
    }

    /// Generates and persists an x25519 keypair for digital signatures.
    /// Returns public Jwk for convenience.
    pub fn gen_x25519_jwk(&mut self) -> Result<Jwk, Box<dyn Error>> {
        let keypair = X25519KeyPair::new().map_err(|_| "Failure to generate X25519 keypair")?;
        let jwk: Jwk = keypair
            .try_into()
            .map_err(|_| "Failure to map to Jwk format")?;
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

trait ToPublic {
    fn to_public(&self) -> Self;
}

impl ToPublic for Jwk {
    fn to_public(&self) -> Self {
        Jwk {
            d: None,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::crate_name;
    use tempdir::TempDir;

    impl KeyStore {
        fn destroy(self) {
            let _ = std::fs::remove_file(self.path);
        }
    }

    #[test]
    fn test_keystore_flow() {
        let location = TempDir::new(&crate_name()).unwrap();
        let factory = KeyStore::factory(location.path().to_str().unwrap());

        let mut store = factory.create();

        let jwk = store.gen_ed25519_jwk().unwrap();
        assert!(store.find_keypair(&jwk).is_some());

        let jwk = store.gen_x25519_jwk().unwrap();
        assert!(store.find_keypair(&jwk).is_some());

        let latest = factory.latest();
        assert!(latest.is_some());

        store.destroy();
    }
}
