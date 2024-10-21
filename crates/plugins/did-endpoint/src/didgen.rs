use database::Repository;
use did_utils::{
    crypto::{Ed25519KeyPair, Generate, PublicKeyFormat, ToMultikey, X25519KeyPair},
    didcore::{AssertionMethod, Authentication, Document, KeyAgreement, KeyFormat, Service},
    jwk::Jwk,
    methods::{DidPeer, Purpose, PurposedKey},
};
use keystore::{KeyStore, Secrets};
use mongodb::bson::doc;
use serde_json::json;
use std::{fmt::Display, path::Path};

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("KeyGenerationError")]
    KeyGenerationError,
    #[error("MissingServerPublicDomain")]
    MissingServerPublicDomain,
    #[error("DidAddressDerivationError")]
    DidAddressDerivationError,
    #[error("PersistenceError")]
    PersistenceError,
    #[error("Generic: {0}")]
    Generic(String),
}

/// Generates keys and forward them for DID generation
pub fn didgen<P>(storage_dirpath: P, server_public_domain: &str) -> Result<Document, Error>
where
    P: AsRef<Path> + Display,
{
    // Generate keys for did:peer generation
    let auth_keys = Ed25519KeyPair::new().unwrap();
    let agreem_keys = X25519KeyPair::new().unwrap();

    let keys = vec![
        PurposedKey {
            purpose: Purpose::Encryption,
            public_key_multibase: agreem_keys.to_multikey(),
        },
        PurposedKey {
            purpose: Purpose::Verification,
            public_key_multibase: auth_keys.to_multikey(),
        },
        PurposedKey {
            purpose: Purpose::Assertion,
            public_key_multibase: auth_keys.to_multikey(),
        },
    ];

    // Build services
    let services = vec![Service {
        id: String::from("#didcomm"),
        service_type: String::from("DIDCommMessaging"),
        service_endpoint: json!({"uri": server_public_domain, "accept": vec!["didcomm/v2"], "routingKeys": Vec::<String>::new()}),
        ..Default::default()
    }];

    // Generate did:peer address
    let did = DidPeer::create_did_peer_2(&keys, &services).unwrap();

    // Generate DID Document
    let diddoc = {
        let resolver = DidPeer::with_format(PublicKeyFormat::Jwk);
        resolver.expand(&did).expect("Could not resolve DID")
    };

    let agreem_keys_jwk: Jwk = agreem_keys.try_into().expect("MediateRequestError");

    let agreem_keys_secret = Secrets {
        id: None,
        kid: match diddoc
            .key_agreement
            .as_ref()
            .unwrap()
            .get(0)
            .unwrap()
            .clone()
        {
            KeyAgreement::Reference(kid) => kid,
            _ => unreachable!(),
        },
        secret_material: agreem_keys_jwk,
    };

    // Create a new KeyStore
    let keystore = KeyStore::new();

    // Store the agreement key in the secrets store
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match keystore.store(agreem_keys_secret).await {
                Ok(_) => {
                    tracing::info!("Successfully stored agreement key.")
                }
                Err(error) => tracing::error!("Error storing agreement key: {:?}", error),
            }
        })
    });

    let auth_keys_jwk: Jwk = auth_keys.try_into().expect("MediateRequestError");

    let auth_keys_secret = Secrets {
        id: None,
        kid: match diddoc
            .authentication
            .as_ref()
            .unwrap()
            .get(0)
            .unwrap()
            .clone()
        {
            Authentication::Reference(kid) => kid,
            _ => unreachable!(),
        },
        secret_material: auth_keys_jwk.clone(),
    };

    // Store the authentication key in the secrets store
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match keystore.store(auth_keys_secret).await {
                Ok(_) => {
                    tracing::info!("Successfully stored authentication key.")
                }
                Err(error) => tracing::error!("Error storing authentication key: {:?}", error),
            }
        })
    });

    let assert_keys_secret = Secrets {
        id: None,
        kid: match diddoc
            .assertion_method
            .as_ref()
            .unwrap()
            .get(0)
            .unwrap()
            .clone()
        {
            AssertionMethod::Reference(kid) => kid,
            _ => unreachable!(),
        },
        secret_material: auth_keys_jwk,
    };

    // Store the assertion key in the secrets store
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match keystore.store(assert_keys_secret).await {
                Ok(_) => {
                    tracing::info!("Successfully stored assertion key.")
                }
                Err(error) => tracing::error!("Error storing assertion key: {:?}", error),
            }
        })
    });

    // Serialize and persist to file
    let pretty_diddoc = serde_json::to_string_pretty(&diddoc).unwrap();

    std::fs::create_dir_all(&storage_dirpath).map_err(|_| Error::PersistenceError)?;
    std::fs::write(format!("{storage_dirpath}/did.json"), pretty_diddoc)
        .map_err(|_| Error::PersistenceError)?;

    tracing::info!("persisted DID document to disk");
    tracing::debug!("successful completion");
    Ok(diddoc)
}

/// Validates the integrity of the persisted diddoc
pub fn validate_diddoc<P>(storage_dirpath: P) -> Result<(), String>
where
    P: AsRef<Path> + Display,
{
    // Validate that did.json exists
    let didpath = format!("{storage_dirpath}/did.json");
    if !Path::new(&didpath).exists() {
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
