pub mod filesystem;

use chacha20poly1305::{aead::generic_array::GenericArray, KeyInit};

use chrono::Utc;
use did_utils::{
    crypto::{Ed25519KeyPair, Generate, ToPublic, X25519KeyPair},
    jwk::Jwk,
};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
};

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
use chacha20poly1305::{
    aead::{Aead, AeadCore, OsRng},
    ChaCha20Poly1305,
};

use log::{debug, info, error}; // Import logging macros
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

// Define a custom error type for keystore operations
#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    #[error("File error: {0}")]
    FileError(std::io::Error),
    #[error("Encryption error: {0}")]
    EncryptionError(chacha20poly1305::Error),
    #[error("Decryption error: {0}")]
    DecryptionError(chacha20poly1305::Error),
}
 

struct FileSystemKeystore {
    key: SecretString, // Store key securely using secrecy crate
    nonce: Vec<u8>,
}

impl FileSystemKeystore {
    fn encrypt(mut self, secret: KeyStore) -> Result<(), Box<dyn Error>> {
        let key = self.key.expose_secret(); // Access key securely
        let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(key.as_bytes()));

        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let path = secret.path();
        let mut keystorefile = File::open(path.clone())?; // Use Result for error handling

        let mut buffer = Vec::new();
        keystorefile.read_to_end(&mut buffer)?; // Use Result for error handling

        let encrypted_key = cipher
            .encrypt(GenericArray::from_slice(&self.nonce), buffer.as_slice())
            .map_err(|err| err).unwrap();
        // Overwrite the file with encrypted keys
        keystorefile.write_all(&encrypted_key)?; // Use Result for error handling

        self.nonce = nonce.to_vec();

        // Overwrite the buffer with zeros to prevent data leakage
        buffer.clear();
        buffer.zeroize();

        // Conditional logging
        debug!("Encryption successful for keystore file: {}", path);

        Ok(())
    }

    fn decrypt(self, secret: KeyStore) -> Result<Vec<u8>, std::io::Error> {
        let key = self.key.expose_secret(); // Access key securely
        let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(key.as_bytes()));

        let path = secret.path();
        let mut keystorefile = File::open(path.clone())?; // Use Result for error handling

        let mut buffer = Vec::new();
        keystorefile.read_to_end(&mut buffer)?; // Use Result for error handling

        let decrypted_key = cipher.decrypt(GenericArray::from_slice(&self.nonce), buffer.as_slice())
            .map_err(|err| err).unwrap();

        // Enhanced redaction: Replace all sensitive characters with asterisks
        let redacted_key = decrypted_key.iter().map(|b| if b.is_ascii_graphic() && !b.is_ascii_whitespace() { '*' as u8 } else { *b }).collect::<Vec<u8>>();

        // Conditional logging with redacted key
        info!("Decryption successful for keystore file: {}, redacted key: {:?}", &path, redacted_key);

        buffer.clear();
        buffer.zeroize();

        Ok(decrypted_key)
    }
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
