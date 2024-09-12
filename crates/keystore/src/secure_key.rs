use crate::crypto::{encrypt_key, decrypt}; // Assuming these functions exist in lib.rs
use secrecy::{Secret, Owned};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

// Defining a struct to hold encrypted keys in memory
#[derive(Serialize, Deserialize)]
struct EncryptedKeyStore {
    keys: HashMap<String, Vec<u8>>,
}

impl EncryptedKeyStore {
    fn from_file(path: &str, password: &str) -> Result<Self, std::io::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let decrypted_data = decrypt(&contents.as_bytes(), password)?;
        let store: EncryptedKeyStore = serde_json::from_slice(&decrypted_data)?;
        Ok(store)
    }

    fn save_to_file(&self, path: &str, password: &str) -> Result<(), std::io::Error> {
        let serialized_data = serde_json::to_string(self)?;
        let encrypted_data = encrypt(&serialized_data.as_bytes(), password)?;

        let mut file = File::create(path)?;
        file.write_all(&encrypted_data)?;
        Ok(())
    }
}

// Singleton-like pattern to access the in-memory key store
static mut ENCRYPTED_KEYS: Option<HashMap<String, Secret<String>>> = None;

fn get_key_store() -> Result<&'static mut HashMap<String, Secret<String>>, std::io::Error> {
    unsafe {
        if ENCRYPTED_KEYS.is_none() {
            let password = prompt_for_password()?;
            let key_store_path = "didcomm-mediator-rs/crates/keystore.json";

            let mut store = match EncryptedKeyStore::from_file(key_store_path, password) {
                Ok(store) => store,
                Err(_) => EncryptedKeyStore { keys: HashMap::new() },
            };

            let mut map = HashMap::new();
            for (key_name, encrypted_key) in store.keys.iter() {
                let decrypted_key = decrypt(encrypted_key, password)?;
                map.insert(key_name.to_owned(), Secret::new(decrypted_key));
            }

            ENCRYPTED_KEYS = Some(map);
        }

        Ok(ENCRYPTED_KEYS.as_mut().unwrap())
    }
}

// Function to prompt for password securely 
fn prompt_for_password() -> Result<String, std::io::Error> {
    // Implement logic to securely prompt for password (e.g., using libraries like rpassword)
    Err(std::io::Error::new(std::io::ErrorKind::Other, "Password prompting not implemented"))
}
// Functions to add and get keys (delegates to lib.rs)
pub fn add_key(key_name: &str, key_value: &Secret<String>, password: &str) -> Result<(), ring::error::RingError> {
    let key_store = get_key_store()?;
    let encrypted_key = encrypt_key(key_value, password)?;
    key_store.insert(key_name.to_owned(), Secret::new(encrypted_key));
    Ok(())
}

pub fn get_key(key_name: &str, password: &str) -> Result<Option<Secret<String>>, ring::error::RingError> {
    let key_store = get_key_store()?;
    let encrypted_key = key_store.get(key_name);
    if let Some(encrypted_key) = encrypted_key {
        decrypt(encrypted_key.as_bytes(), password)
    } else {
        Ok(None)
    }
}

// Function to persist all encrypted keys before program termination (delegates to lib.rs)
pub fn persist_keys(path: &str, password: &str) -> Result<(), std::io::Error> {
    let key_store = get_key_store()?;
    let mut store = EncryptedKeyStore { keys: HashMap::new() };
    for (key_name, key_value) in key_store.iter() {
        let encrypted_key = encrypt_key(key_value, password)?;
        store.keys.insert(key_name.to_owned(), encrypted_key);
    }

    store.save_to_file(path, password)?;
    Ok(())
}