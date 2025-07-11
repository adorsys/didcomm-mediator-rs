use crate::persistence::MediatorDidDocument;
use database::{Repository, RepositoryError};
use did_utils::{
    crypto::{Ed25519KeyPair, Generate, PublicKeyFormat, ToMultikey, X25519KeyPair},
    didcore::{Document, KeyFormat, Service, VerificationMethodType},
    jwk::Jwk,
    methods::{DidPeer, Purpose, PurposedKey},
};
use keystore::Keystore;
use mongodb::bson::doc;
use serde_json::json;
use tokio::{runtime::Handle, task};

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
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Generates keys and forward them for DID generation
pub fn didgen<R>(
    server_public_domain: &str,
    keystore: &Keystore,
    repository: &R,
) -> Result<Document, Error>
where
    R: Repository<MediatorDidDocument> + ?Sized,
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

    // Persist the diddoc
    persist_did_document(&diddoc, repository)?;

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

fn store_key(
    key: Jwk,
    diddoc: &Document,
    field: &Option<Vec<VerificationMethodType>>,
    keystore: &Keystore,
) -> Result<(), Error> {
    // Extract key ID from the DID document
    let kid = match field.as_ref().unwrap()[0].clone() {
        VerificationMethodType::Reference(kid) => kid,
        VerificationMethodType::Embedded(method) => method.id,
    };
    let kid = crate::util::handle_vm_id(&kid, diddoc);

    // Store the secret in the keystore
    task::block_in_place(move || {
        Handle::current().block_on(async move { keystore.store(&kid, &key).await })
    })
    .map(|_| tracing::info!("Successfully stored secret."))
    .map_err(|err| {
        tracing::error!("Error storing secret: {err:?}");
        Error::KeyStoringError
    })?;

    Ok(())
}

fn persist_did_document<R>(diddoc: &Document, repository: &R) -> Result<(), Error>
where
    R: Repository<MediatorDidDocument> + ?Sized,
{
    let doc = MediatorDidDocument {
        id: None,
        diddoc: diddoc.clone(),
    };
    task::block_in_place(move || {
        Handle::current().block_on(async move { repository.store(doc).await })
    })?;
    Ok(())
}

/// Validates the integrity of the persisted diddoc
pub fn validate_diddoc<R>(keystore: &Keystore, repository: &R) -> Result<(), String>
where
    R: Repository<MediatorDidDocument> + ?Sized,
{
    // Find from repository
    let result = task::block_in_place(move || {
        Handle::current().block_on(async move { repository.find_one_by(doc! {}).await })
    });

    let diddoc_entity = result
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Missing did.json from repository".to_string())?;

    let diddoc: Document = serde_json::from_value(json!(diddoc_entity.diddoc))
        .map_err(|e| format!("Failed to deserialize DID document: {e}"))?;

    // Validate that verification methods are present
    if let Some(verification_methods) = &diddoc.verification_method {
        // Validate that each key is present in the keystore
        for method in verification_methods {
            let pubkey = method
                .public_key
                .as_ref()
                .ok_or(String::from("Missing key"))?;
            let kid = crate::util::handle_vm_id(&method.id, &diddoc);
            match pubkey {
                KeyFormat::Jwk(_) => validate_key(&kid, keystore)?,
                _ => return Err(String::from("Unsupported key format")),
            };
        }
    }

    Ok(())
}

fn validate_key(kid: &str, keystore: &Keystore) -> Result<(), String> {
    // Validate that the key exists
    task::block_in_place(|| {
        Handle::current().block_on(async move { keystore.retrieve::<Jwk>(kid).await })
    })
    .map_err(|_| String::from("Error fetching secret"))?
    .ok_or_else(|| String::from("Mismatch or missing secret"))?;

    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::persistence::tests::MockDidDocumentRepository;

    pub(crate) fn setup() -> Jwk {
        serde_json::from_str(
            r##"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "PuG2L5um-tAnHlvT29gTm9Wj9fZca16vfBCPKsHB5cA",
                "d": "af7bypYk00b4sVpSDit1gMGvnmlQI52X4pFBWYXndUA"
            }"##,
        )
        .unwrap()
    }

    // Verifies that the didgen function returns a DID document.
    // Does not validate the DID document.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_did_generation() {
        let kid = "did:peer:123#key-1".to_string();
        let repository = MockDidDocumentRepository::new();
        let mock_keystore = Keystore::with_mock_configs(vec![(kid, setup())]);

        let result = didgen("https://example.com", &mock_keystore, &repository);

        assert!(result.is_ok());
        let stored_diddoc = repository
            .find_one_by(doc! {})
            .await
            .unwrap()
            .expect("diddoc should be stored");

        let result_diddoc: Document = serde_json::from_value(json!(result.unwrap())).unwrap();
        let stored_diddoc: Document = serde_json::from_value(json!(stored_diddoc.diddoc)).unwrap();
        assert_eq!(result_diddoc.id, stored_diddoc.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_did_validation() {
        let kid = "did:peer:123#key-1";
        let repository = MockDidDocumentRepository::new();
        let mock_keystore = Keystore::with_mock_configs(vec![(kid.to_string(), setup())]);

        // Mock read from store
        let diddoc: Document = serde_json::from_str(
            r##"{
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
            }"##,
        )
        .unwrap();
        repository
            .store(MediatorDidDocument { id: None, diddoc })
            .await
            .unwrap();

        let result = validate_diddoc(&mock_keystore, &repository);

        assert!(result.is_ok());
    }
}
