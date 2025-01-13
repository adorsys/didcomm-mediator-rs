use did_utils::{
    crypto::{Ed25519KeyPair, Generate, PublicKeyFormat, ToMultikey, X25519KeyPair},
    didcore::{Document, KeyFormat, Service, VerificationMethodType},
    jwk::Jwk,
    methods::{DidPeer, Purpose, PurposedKey},
};
use filesystem::FileSystem;
use keystore::{Secrets, SecureRepository, WrapSecret};
use mongodb::bson::doc;
use serde_json::json;
use std::path::Path;
use tokio::{runtime::Handle, task};

use crate::util;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Key Generation Error")]
    KeyGenerationError,
    #[error("Key Storing Error")]
    KeyStoringError,
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
    master_key: [u8; 32],
) -> Result<Document, Error>
where
    K: SecureRepository<WrapSecret>,
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
    store_key(
        auth_keys_jwk,
        &diddoc,
        &diddoc.authentication,
        keystore,
        master_key,
    )?;
    store_key(
        agreem_keys_jwk,
        &diddoc,
        &diddoc.key_agreement,
        keystore,
        master_key,
    )?;

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
    master_key: [u8; 32],
) -> Result<(), Error>
where
    S: SecureRepository<WrapSecret>,
{
    // Extract key ID from the DID document
    let kid = match field.as_ref().unwrap()[0].clone() {
        VerificationMethodType::Reference(kid) => kid,
        VerificationMethodType::Embedded(method) => method.id,
    };
    let kid = util::handle_vm_id(&kid, diddoc);

    // Create Secrets for the key
    let secret = Secrets {
        id: None,
        kid: kid.into_owned(),
        secret_material: key,
    };

    // Store the secret in the keystore
    task::block_in_place(move || {
        Handle::current()
            .block_on(async move { keystore.secure_store(secret.into(), master_key).await })
    })
    .map(|_| tracing::info!("Successfully stored secret."))
    .map_err(|err| {
        tracing::error!("Error storing secret: {err:?}");
        Error::KeyStoringError
    })?;

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
        .create_dir_all(storage_dirpath)
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
    master_key: [u8; 32],
) -> Result<(), String>
where
    K: SecureRepository<WrapSecret>,
    F: FileSystem,
{
    cfg_if::cfg_if! {
        if #[cfg(not(test))] {
            // Validate that did.json exists
            let didpath = storage_dirpath.join("did.json");
            if !didpath.exists() {
                return Err(String::from("Missing did.json"));
            };
        }
    }

    // Load the DID document
    let diddoc: Document = filesystem
        .read_to_string(&storage_dirpath.join("did.json"))
        .map_err(|_| String::from("Unreadable did.json"))
        .and_then(|content| {
            serde_json::from_str(&content).map_err(|_| String::from("Unparseable did.json"))
        })?;

    // Validate the keys in the DID document
    if let Some(verification_methods) = &diddoc.verification_method {
        for method in verification_methods {
            let pubkey = method
                .public_key
                .as_ref()
                .ok_or(String::from("Missing key"))?;
            let kid = util::handle_vm_id(&method.id, &diddoc);
            match pubkey {
                KeyFormat::Jwk(_) => validate_key(&kid, keystore, master_key)?,
                _ => return Err(String::from("Unsupported key format")),
            };
        }
    }

    Ok(())
}

fn validate_key<K>(kid: &str, keystore: &K, master_key: [u8; 32]) -> Result<(), String>
where
    K: SecureRepository<WrapSecret>,
{
    // Validate that the key exists
    task::block_in_place(|| {
        Handle::current()
            .block_on(async move { keystore.find_key_by(doc! { "kid": kid }, master_key).await })
    })
    .map_err(|_| String::from("Error fetching secret"))?
    .ok_or_else(|| String::from("Mismatch or missing secret"))?;

    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use database::RepositoryError;
    use filesystem::FileSystem;
    use mockall::{
        mock,
        predicate::{self, *},
    };
    use mongodb::bson::Document as BsonDocument;
    use std::io::Result as IoResult;

    // Mock the FileSystem trait
    mock! {
        pub FileSystem {}
        impl FileSystem for FileSystem {
            fn read_to_string(&self, path: &Path) -> IoResult<String>;
            fn write(&mut self, path: &Path, content: &str) -> IoResult<()>;
            fn read_dir_files(&self, path: &Path) -> IoResult<Vec<String>>;
            fn create_dir_all(&mut self, path: &Path) -> IoResult<()>;
            fn write_with_lock(&self, path: &Path, content: &str) -> IoResult<()>;
        }
    }

    // Mock the Repository trait
    mock! {
        pub Keystore {}
        #[async_trait::async_trait]
        impl SecureRepository<WrapSecret> for Keystore {
            fn get_collection(&self) -> std::sync::Arc<tokio::sync::RwLock<mongodb::Collection<keystore::WrapSecret>>> ;
            async fn secure_store(&self, entity: keystore::WrapSecret, master_key: [u8; 32]) -> Result<WrapSecret, RepositoryError>;
          async fn find_key_by(&self, filter: BsonDocument, master_key: [u8; 32]) -> Result<Option<Secrets>, RepositoryError>;
        }
    }

    pub(crate) fn setup() -> Secrets {
        serde_json::from_str(
            r##"{
                "kid": "did:peer:123#key-1",
                "secret_material": {
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "x": "PuG2L5um-tAnHlvT29gTm9Wj9fZca16vfBCPKsHB5cA",
                    "d": "af7bypYk00b4sVpSDit1gMGvnmlQI52X4pFBWYXndUA"
                }
            }"##,
        )
        .unwrap()
    }

    // Verifies that the didgen function returns a DID document.
    // Does not validate the DID document.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_did_generation() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_keystore = MockKeystore::new();
        let path = Path::new("/mock/dir");
        let secret = setup();

        // Mock the file system write call
        mock_fs
            .expect_create_dir_all()
            .with(predicate::eq(path))
            .times(1)
            .returning(|_| Ok(()));

        mock_fs.expect_write().times(1).returning(|_, _| Ok(()));

        // Mock keystore save key
        mock_keystore
            .expect_secure_store()
            .times(2)
            .returning(move |_, _master_key| {
                let secrets: WrapSecret = secret.clone().into();
                Ok(secrets)
            });

        // master_key
        let master_key = [0; 32];

        let result = didgen(
            &path,
            "https://example.com",
            &mock_keystore,
            &mut mock_fs,
            master_key,
        );

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_did_validation() {
        let mut mock_fs = MockFileSystem::new();
        let mock_keystore = MockKeystore::new();
        let path = Path::new("/mock/dir");

        // Mock read from filesystem
        mock_fs
            .expect_read_to_string()
            .withf(|path| path.ends_with("did.json"))
            .times(1)
            .returning(|_| {
                Ok(r##"{
                        "@context": ["https://www.w3.org/ns/did/v1"],
                        "id": "did:peer:123",
                        "verificationMethod": [
                            {
                                "id": "#key-1",
                                "type": "JsonWebKey2020",
                                "controller": "did:peer:123",
                                "publicKeyJwk": {
                                    "kty": "OKP",
                                    "crv": "Ed25519",
                                    "x": "PuG2L5um-tAnHlvT29gTm9Wj9fZca16vfBCPKsHB5cA"
                                }
                            }
                        ]
                    }"##
                .to_string())
            });

        // Mock keystore fetch
        // dummy master_key
        let master_key = [0; 32];
        let _ = mock_keystore.find_key_by(doc! {}, master_key);
        // .with(predicate::function(|filter: &BsonDocument| {
        //     filter.contains_key("kid")
        // }))
        // .returning(move |_| Ok(Some(secret.clone())));
        // dummy master_key

        let result = validate_diddoc(&path, &mock_keystore, &mut mock_fs, master_key);

        assert!(result.is_ok());
    }
}
