use database::Repository;
use did_utils::{
    crypto::{Ed25519KeyPair, Generate, PublicKeyFormat, ToMultikey, X25519KeyPair},
    didcore::{Document, KeyFormat, Service, VerificationMethodType},
    jwk::Jwk,
    methods::{DidPeer, Purpose, PurposedKey},
};
use filesystem::FileSystem;
use keystore::Secrets;
use mongodb::bson::doc;
use serde_json::json;
use std::path::Path;
use tokio::{runtime::Handle, task};

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
    storage_dirpath: &Path,
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
    let diddoc = generate_did_document(keys, services)?;

    // Convert keys to JWK format
    let auth_keys_jwk: Jwk = auth_keys
        .try_into()
        .map_err(|_| Error::KeyConversionError)?;
    let agreem_keys_jwk: Jwk = agreem_keys
        .try_into()
        .map_err(|_| Error::KeyConversionError)?;

    // Store authentication and agreement keys in the keystore.
    store_key(auth_keys_jwk, &diddoc, &diddoc.authentication, keystore)?;
    store_key(agreem_keys_jwk, &diddoc, &diddoc.key_agreement, keystore)?;

    // Serialize DID document and persist to filesystem
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
    keys: Vec<PurposedKey>,
    services: Vec<Service>,
) -> Result<Document, Error> {
    let did =
        DidPeer::create_did_peer_2(&keys, &services).map_err(|_| Error::DidGenerationError)?;
    let resolver = DidPeer::with_format(PublicKeyFormat::Jwk);
    resolver.expand(&did).map_err(|_| Error::DidGenerationError)
}

fn store_key<S>(
    key: Jwk,
    diddoc: &Document,
    field: &Option<Vec<VerificationMethodType>>,
    keystore: &S,
) -> Result<(), Error>
where
    S: Repository<Secrets>,
{
    // Extract key ID from the DID document
    let kid = match field.as_ref().unwrap().get(0).unwrap().clone() {
        VerificationMethodType::Reference(kid) => kid,
        VerificationMethodType::Embedded(method) => method.id,
    };
    let kid = format!(
        "{}{}",
        diddoc.also_known_as.as_ref().unwrap().get(0).unwrap(),
        kid
    );

    // Create Secrets for the key
    let secret = Secrets {
        id: None,
        kid,
        secret_material: key,
    };

    // Store the secret in the keystore
    task::block_in_place(move || {
        Handle::current().block_on(async move {
            match keystore.store(secret).await {
                Ok(_) => {
                    tracing::info!("Successfully stored secret.")
                }
                Err(error) => tracing::error!("Error storing secret: {:?}", error),
            }
        })
    });

    Ok(())
}

fn persist_did_document<F>(
    storage_dirpath: &Path,
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
        .create_dir_all(&storage_dirpath)
        .map_err(|_| Error::PersistenceError)?;
    filesystem
        .write(&storage_dirpath.join("did.json"), &pretty_diddoc)
        .map_err(|_| Error::PersistenceError)?;

    Ok(())
}

/// Validates the integrity of the persisted diddoc
pub fn validate_diddoc<K, F>(
    storage_dirpath: &Path,
    keystore: &K,
    filesystem: &mut F,
) -> Result<(), String>
where
    K: Repository<Secrets>,
    F: FileSystem,
{
    // Validate that did.json exists
    let didpath = storage_dirpath.join("did.json");
    if !didpath.exists() {
        return Err(String::from("Missing did.json"));
    };

    // Load the DID document
    let diddoc: Document = filesystem
        .read_to_string(&didpath)
        .map_err(|_| String::from("Unreadable did.json"))
        .and_then(|content| {
            serde_json::from_str(&content).map_err(|_| String::from("Unparseable did.json"))
        })?;

    // Validate the keys in the DID document
    for method in diddoc.verification_method.unwrap_or(vec![]) {
        let pubkey = method.public_key.ok_or(String::from("Missing key"))?;
        let kid = format!(
            "{}{}",
            diddoc.also_known_as.as_ref().unwrap()[0],
            method.id
        );
        match pubkey {
            KeyFormat::Jwk(_) => validate_key(&kid, keystore)?,
            _ => return Err(String::from("Unsupported key format")),
        };
    }

    Ok(())
}

fn validate_key<K>(kid: &str, keystore: &K) -> Result<(), String>
where
    K: Repository<Secrets>,
{
    // Validate that the key exists
    task::block_in_place(|| {
        Handle::current().block_on(async move { keystore.find_one_by(doc! { "kid": kid }).await })
    })
    .map_err(|_| String::from("Error fetching secret"))?
    .ok_or_else(|| String::from("Mismatch or missing secret"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::dotenv_flow_read;
    use filesystem::MockFileSystem;
    use keystore::tests::MockKeyStore;

    fn setup() -> (String, String) {
        let storage_dirpath = "../../filesystem/test/storage".to_string();

        let server_public_domain = dotenv_flow_read("SERVER_PUBLIC_DOMAIN").unwrap();

        (storage_dirpath, server_public_domain)
    }

    // Verifies that the didgen function returns a DID document.
    // Does not validate the DID document.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_didgen_creation_and_validation() {
        let (storage_dirpath, server_public_domain) = setup();
        let mut filesystem = MockFileSystem;
        let keystore = MockKeyStore::new(vec![]);

        let diddoc = didgen(
            storage_dirpath.as_ref(),
            &server_public_domain,
            &keystore,
            &mut filesystem,
        );
        assert!(diddoc.is_ok());

        assert!(validate_diddoc(&storage_dirpath.as_ref(), &keystore, &mut filesystem).is_ok());
    }
}
