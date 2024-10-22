pub mod filesystem;

use chrono::Utc;
use did_utils::{
    crypto::{Ed25519KeyPair, Generate, ToPublic, X25519KeyPair},
    jwk::Jwk,
};
use std::error::Error;

use crate::filesystem::FileSystem;

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

pub struct KeyStore<'a> {
    fs: &'a mut dyn FileSystem,
    dirpath: String,
    filename: String,
    keys: Vec<Jwk>,
}

impl<'a> KeyStore<'a> {
    /// Constructs file-based key-value store.
    pub fn new(fs: &'a mut dyn FileSystem, storage_dirpath: &str) -> Self {
        Self {
            fs,
            dirpath: format!("{storage_dirpath}/keystore"),
            filename: format!("{}.json", Utc::now().timestamp()),
            keys: vec![],
        }
    }

    /// Returns latest store on disk, if any.
    pub fn latest(
        fs: &'a mut dyn FileSystem,
        storage_dirpath: &str,
    ) -> Result<Self, KeyStoreError> {
        let dirpath = format!("{storage_dirpath}/keystore");

        // Read directory
        let paths = fs
            .read_dir_files(&dirpath)
            .map_err(KeyStoreError::IoError)?;

        // Collect paths and associated timestamps of files inside `dir`
        let mut collected: Vec<(String, i32)> = vec![];
        for path in paths {
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
        let content = fs.read_to_string(path).map_err(KeyStoreError::IoError)?;
        let keys = serde_json::from_str::<Vec<Jwk>>(&content).map_err(KeyStoreError::ParseError)?;

        let filename = path
            .trim_start_matches(&format!("{}/", &dirpath))
            .to_string();

        Ok(KeyStore {
            fs,
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
    fn persist(&mut self) -> Result<(), KeyStoreError> {
        self.fs
            .create_dir_all(&self.dirpath)
            .map_err(KeyStoreError::IoError)?;
        self.fs
            .write(
                &self.path(),
                &serde_json::to_string_pretty(&self.keys).map_err(KeyStoreError::SerdeError)?,
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Result as IoResult;

    #[derive(Default)]
    struct MockFileSystem {
        stream: String,
    }

    impl FileSystem for MockFileSystem {
        fn read_to_string(&self, _path: &str) -> IoResult<String> {
            Ok(self.stream.clone())
        }

        fn write(&mut self, _path: &str, content: &str) -> IoResult<()> {
            self.stream = content.to_string();
            Ok(())
        }

        fn read_dir_files(&self, _path: &str) -> IoResult<Vec<String>> {
            Ok(vec!["/keystore/12345.json".to_string()])
        }

        fn create_dir_all(&mut self, _path: &str) -> IoResult<()> {
            Ok(())
        }

        fn write_with_lock(&self, _path: &str, _content: &str) -> IoResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_keystore_flow() {
        let mut mock_fs = MockFileSystem::default();
        let mut store = KeyStore::new(&mut mock_fs, "");

        let jwk = store.gen_ed25519_jwk().unwrap();
        assert!(store.find_keypair(&jwk).is_some());

        let jwk = store.gen_x25519_jwk().unwrap();
        assert!(store.find_keypair(&jwk).is_some());

        let latest = KeyStore::latest(&mut mock_fs, "");
        assert!(latest.is_ok());
    }
}
