use chrono::Utc;
use did_utils::{
    crypto::{ed25519::Ed25519KeyPair, traits::Generate, x25519::X25519KeyPair},
    key_jwk::{ec::Ec, jwk::Jwk, key::Key, oct::Oct, okp::Okp, rsa::Rsa, secret::Secret},
};
use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("failure to convert to JWK format")]
    JwkConversionError,
    #[error("failure to generate key pair")]
    KeyPairGenerationError,
    #[error("ioerror: {0}")]
    IoError(std::io::Error),
    #[error("non compliant")]
    NonCompliant,
    #[error("not found")]
    NotFound,
    #[error("parse error")]
    ParseError(serde_json::Error),
    #[error("serde error")]
    SerdeError(serde_json::Error),
}

pub struct KeyStore {
    dirpath: String,
    filename: String,
    keys: Vec<Jwk>,
}

impl KeyStore {
    /// Constructs file-based key-value store.
    pub fn new(storage_dirpath: &str) -> Self {
        Self {
            dirpath: format!("{storage_dirpath}/keystore"),
            filename: format!("{}.json", Utc::now().timestamp()),
            keys: vec![],
        }
    }

    /// Returns latest store on disk, if any.
    pub fn latest(storage_dirpath: &str) -> Result<Self, KeyStoreError> {
        let dirpath = format!("{storage_dirpath}/keystore");

        // Read directory
        let dir = std::fs::read_dir(&dirpath).map_err(KeyStoreError::IoError)?;

        // Collect paths and associated timestamps of files inside `dir`
        let mut collected: Vec<(String, i32)> = vec![];
        for file in dir {
            let file = file.map_err(KeyStoreError::IoError)?;
            let path = file
                .path()
                .to_str()
                .ok_or(KeyStoreError::NonCompliant)?
                .to_string();

            if path.ends_with(".json") {
                let stamp: i32 = path
                    .trim_start_matches(&format!("{}/", &dirpath))
                    .trim_end_matches(".json")
                    .parse()
                    .map_err(|_| KeyStoreError::NonCompliant)?;

                collected.push((path, stamp));
            }
        }

        // Select file with highest timestamp as latest keystore
        let file = collected
            .iter()
            .max_by_key(|(_, stamp)| stamp)
            .map(|(path, _)| path);

        let path = file.ok_or(KeyStoreError::NotFound)?;
        let content = std::fs::read_to_string(path).map_err(KeyStoreError::IoError)?;
        let keys = serde_json::from_str::<Vec<Jwk>>(&content).map_err(KeyStoreError::ParseError)?;

        let filename = path
            .trim_start_matches(&format!("{}/", &dirpath))
            .to_string();

        Ok(KeyStore {
            dirpath,
            filename,
            keys,
        })
    }

    /// Gets path
    pub fn path(&self) -> String {
        format!("{}/{}", self.dirpath, self.filename)
    }

    /// Persists store on disk
    fn persist(&self) -> Result<(), KeyStoreError> {
        std::fs::create_dir_all(&self.dirpath).map_err(KeyStoreError::IoError)?;
        std::fs::write(
            self.path(),
            serde_json::to_string_pretty(&self.keys).map_err(KeyStoreError::SerdeError)?,
        )
        .map_err(KeyStoreError::IoError)
    }

    /// Searches keypair given public key
    pub fn find_keypair(&self, pubkey: &Jwk) -> Option<Jwk> {
        self.keys.iter().find(|k| &k.to_public() == pubkey).cloned()
    }

    /// Generates and persists an ed25519 keypair for digital signatures.
    /// Returns public Jwk for convenience.
    pub fn gen_ed25519_jwk(&mut self) -> Result<Jwk, Box<dyn Error>> {
        let keypair = Ed25519KeyPair::new().map_err(|_| KeyStoreError::KeyPairGenerationError)?;
        let jwk: Jwk = keypair
            .try_into()
            .map_err(|_| KeyStoreError::JwkConversionError)?;
        let pub_jwk = jwk.to_public();

        self.keys.push(jwk);
        self.persist()?;

        Ok(pub_jwk)
    }

    /// Generates and persists an x25519 keypair for digital signatures.
    /// Returns public Jwk for convenience.
    pub fn gen_x25519_jwk(&mut self) -> Result<Jwk, KeyStoreError> {
        let keypair = X25519KeyPair::new().map_err(|_| KeyStoreError::KeyPairGenerationError)?;
        let jwk: Jwk = keypair
            .try_into()
            .map_err(|_| KeyStoreError::JwkConversionError)?;
        let pub_jwk = jwk.to_public();

        self.keys.push(jwk);
        self.persist()?;

        Ok(pub_jwk)
    }
}

pub trait ToPublic {
    fn to_public(&self) -> Self;
}

impl ToPublic for Jwk {
    fn to_public(&self) -> Self {
        let public_key = match &self.key {
            Key::Ec(ec) => Key::Ec(Ec {
                crv: ec.crv,
                x: ec.x.clone(),
                y: ec.y.clone(),
                d: None,
            }),
            Key::Rsa(rsa) => Key::Rsa(Rsa {
                prv: None,
                e: rsa.e.clone(),
                n: rsa.n.clone(),
            }),
            Key::Oct(_oct) => Key::Oct(Oct {
                k: Secret::default(),
            }),
            Key::Okp(okp) => Key::Okp(Okp {
                d: None,
                crv: okp.crv,
                x: okp.x.clone(),
            }),
            _ => {
                return self.clone();
            }
        };

        Jwk {
            key: public_key,
            prm: self.prm.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::dotenv_flow_read;

    fn setup() -> String {
        dotenv_flow_read("STORAGE_DIRPATH")
            .map(|p| format!("{}/{}", p, uuid::Uuid::new_v4()))
            .unwrap()
    }

    fn cleanup(storage_dirpath: &str) {
        std::fs::remove_dir_all(storage_dirpath).unwrap();
    }

    #[test]
    fn test_keystore_flow() {
        let storage_dirpath = setup();

        let mut store = KeyStore::new(&storage_dirpath);

        let jwk = store.gen_ed25519_jwk().unwrap();
        assert!(store.find_keypair(&jwk).is_some());

        let jwk = store.gen_x25519_jwk().unwrap();
        assert!(store.find_keypair(&jwk).is_some());

        let latest = KeyStore::latest(&storage_dirpath);
        assert!(latest.is_ok());

        cleanup(&storage_dirpath);
    }
}
