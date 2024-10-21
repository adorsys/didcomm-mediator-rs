use database::Repository;
use did_utils::{
    crypto::{Ed25519KeyPair, Generate, PublicKeyFormat, ToMultikey, X25519KeyPair},
    didcore::{Document, KeyFormat, Service, VerificationMethodType},
    jwk::Jwk,
    methods::{DidPeer, Purpose, PurposedKey},
};
use filesystem::FileSystem;
use keystore::{KeyStore, Secrets};
use mongodb::bson::doc;
use serde_json::json;
use std::{fmt::Display, path::Path};

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Key Generation Error")]
    KeyGenerationError,
    #[error("Key Conversion Error")]
    KeyConversionError,
    #[error("DID Generation Error")]
    DidGenerationError,
    #[error("PersistenceError")]
    PersistenceError,
    #[error("Generic: {0}")]
    Generic(String),
}

/// Generates keys and forward them for DID generation
pub fn didgen<K, F>(
    storage_dirpath: &str,
    server_public_domain: &str,
    keystore: &K,
    filesystem: &mut F,
) -> Result<Document, Error>
where
    K: Repository<Secrets>,
    F: FileSystem,
{
    // Generate keys for did:peer generation
    let (auth_keys, agreem_keys) = generate_keys()?;

    let keys = vec![
        PurposedKey {
            purpose: Purpose::Verification,
            public_key_multibase: auth_keys.to_multikey(),
        },
        PurposedKey {
            purpose: Purpose::Encryption,
            public_key_multibase: agreem_keys.to_multikey(),
        },
    ];

    // Build services
    let services = vec![Service {
        id: String::from("#didcomm"),
        service_type: String::from("DIDCommMessaging"),
        service_endpoint: json!({
            "uri": server_public_domain,
            "accept": vec!["didcomm/v2"],
            "routingKeys": Vec::<String>::new()}),
        ..Default::default()
    }];

    // Generate DID Document
    let diddoc = generate_did_document(&keys, &services)?;

    // Convert keys to JWK format
    let auth_keys_jwk: Jwk = auth_keys
        .try_into()
        .map_err(|_| Error::KeyConversionError)?;
    let agreem_keys_jwk: Jwk = agreem_keys
        .try_into()
        .map_err(|_| Error::KeyConversionError)?;

    // Store authentication and agreement keys
    store_key(auth_keys_jwk, &diddoc.authentication, keystore)?;
    store_key(agreem_keys_jwk, &diddoc.key_agreement, keystore)?;

    // Step 5: Serialize DID document and persist to file
    persist_did_document(storage_dirpath, &diddoc, filesystem)?;

    tracing::info!("DID generation and persistence successful");
    Ok(diddoc)
}

fn generate_keys() -> Result<(Ed25519KeyPair, X25519KeyPair), Error> {
    let auth_keys = Ed25519KeyPair::new().map_err(|_| Error::KeyGenerationError)?;
    let agreem_keys = X25519KeyPair::new().map_err(|_| Error::KeyGenerationError)?;
    Ok((auth_keys, agreem_keys))
}

fn generate_did_document(
    keys: &Vec<PurposedKey>,
    services: &Vec<Service>,
) -> Result<Document, Error> {
    let did = DidPeer::create_did_peer_2(keys, services).map_err(|_| Error::DidGenerationError)?;
    let resolver = DidPeer::with_format(PublicKeyFormat::Jwk);
    resolver.expand(&did).map_err(|_| Error::DidGenerationError)
}

fn store_key<S>(
    key: Jwk,
    field: &Option<Vec<VerificationMethodType>>,
    keystore: &S,
) -> Result<(), Error>
where
    S: Repository<Secrets>,
{
    // Extract key ID from the DID document
    let kid = match field.as_ref().unwrap().get(0).unwrap().clone() {
        VerificationMethodType::Reference(kid) => kid,
        _ => return Err(Error::Generic("Unable to extract key ID".to_owned())),
    };

    // Create Secrets for the key
    let secret = Secrets {
        id: None,
        kid,
        secret_material: key,
    };

    // Store the secret in the keystore
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match keystore.store(secret).await {
                Ok(_) => {
                    tracing::info!("Successfully stored agreement key.")
                }
                Err(error) => tracing::error!("Error storing agreement key: {:?}", error),
            }
        })
    });

    Ok(())
}

fn persist_did_document<F>(
    storage_dirpath: &str,
    diddoc: &Document,
    filesystem: &mut F,
) -> Result<(), Error>
where
    F: FileSystem,
{
    // Serialize DID document
    let pretty_diddoc = serde_json::to_string_pretty(&diddoc).unwrap();

    // Create directory and write the DID document
    filesystem
        .create_dir_all(storage_dirpath)
        .map_err(|_| Error::PersistenceError)?;
    filesystem
        .write(&format!("{}/did.json", storage_dirpath), &pretty_diddoc)
        .map_err(|_| Error::PersistenceError)?;

    Ok(())
}

/// Validates the integrity of the persisted diddoc
pub fn validate_diddoc<P>(storage_dirpath: P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    // Validate that did.json exists
    let didpath = storage_dirpath.as_ref().join("did.json");
    if !didpath.exists() {
        return Err(String::from("Missing did.json"));
    };

    // Ensure the validity of the persisted diddoc
    let diddoc: Document = match std::fs::read_to_string(didpath) {
        Ok(content) => {
            serde_json::from_str(&content).map_err(|_| String::from("Unparseable did.json"))?
        }
        Err(_) => return Err(String::from("Unreadable did.json")),
    };

    for method in diddoc.verification_method.unwrap_or(vec![]) {
        let pubkey = method.public_key.ok_or(String::from("Missing key"))?;
        let _ = match pubkey {
            KeyFormat::Jwk(jwk) => jwk,
            _ => return Err(String::from("Unsupported key format")),
        };

        let keystore = KeyStore::get();

        let secret = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { keystore.find_one_by(doc! { "kid": method.id }).await })
        });

        secret
            .map_err(|_| String::from("Error fetching secret"))?
            .ok_or_else(|| String::from("Mismatch or missing secret"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::dotenv_flow_read;

    fn setup() -> (String, String) {
        let storage_dirpath = dotenv_flow_read("STORAGE_DIRPATH")
            .map(|p| format!("{}/{}", p, uuid::Uuid::new_v4()))
            .unwrap();

        let server_public_domain = dotenv_flow_read("SERVER_PUBLIC_DOMAIN").unwrap();

        (storage_dirpath, server_public_domain)
    }

    fn cleanup(storage_dirpath: &str) {
        std::fs::remove_dir_all(storage_dirpath).unwrap();
    }

    // Verifies that the didgen function returns a DID document.
    // Does not validate the DID document.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_didgen() {
        dotenv_flow::from_filename("../../../.env").ok();
        let (storage_dirpath, server_public_domain) = setup();

        let diddoc = didgen(&storage_dirpath, &server_public_domain);
        assert!(diddoc.is_ok());

        cleanup(&storage_dirpath);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_validate_diddoc() {
        dotenv_flow::from_filename("../../../.env").ok();
        let (storage_dirpath, server_public_domain) = setup();

        didgen(&storage_dirpath, &server_public_domain).unwrap();
        assert!(validate_diddoc(&storage_dirpath).is_ok());

        cleanup(&storage_dirpath);
    }
}
