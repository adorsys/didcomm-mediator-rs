use crate::repository::entity::Secrets;
use async_trait::async_trait;
use database::Repository;
use did_utils::{
    crypto::PublicKeyFormat,
    didcore::Document,
    methods::{DidKey, DidPeer},
};
use didcomm::{
    did::{DIDDoc, DIDResolver},
    error::{Error, ErrorKind, Result},
    secrets::{Secret, SecretsResolver},
};
use mongodb::bson::doc;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct LocalDIDResolver {
    diddoc: DIDDoc,
}

impl LocalDIDResolver {
    pub fn new(server_diddoc: &Document) -> Self {
        Self {
            diddoc: serde_json::from_value(json!(server_diddoc))
                .expect("Should easily convert between documents representations"),
        }
    }
}

#[async_trait]
impl DIDResolver for LocalDIDResolver {
    async fn resolve(&self, did: &str) -> Result<Option<DIDDoc>> {
        if did == self.diddoc.id {
            return Ok(Some(self.diddoc.clone()));
        }

        if did.starts_with("did:key") {
            Ok(DidKey::new_full(true, PublicKeyFormat::Jwk)
                .expand(did)
                .map(|d| {
                    Some(
                        serde_json::from_value(json!(d))
                            .expect("Should easily convert between documents representations"),
                    )
                })
                .map_err(|e| Error::new(ErrorKind::DIDNotResolved, e))?)
        } else if did.starts_with("did:peer") {
            Ok(DidPeer::with_format(PublicKeyFormat::Jwk)
                .expand(did)
                .map(|d| {
                    Some(
                        serde_json::from_value(json!(d))
                            .expect("Should easily convert between documents representations"),
                    )
                })
                .map_err(|e| Error::new(ErrorKind::DIDNotResolved, e))?)
        } else {
            Err(Error::msg(
                ErrorKind::Unsupported,
                "Unsupported DID".to_string(),
            ))
        }
    }
}

#[derive(Clone)]
pub struct LocalSecretsResolver {
    secrets_repository: Arc<dyn Repository<Secrets>>,
}

impl LocalSecretsResolver {
    pub fn new(secrets_repository: Arc<dyn Repository<Secrets>>) -> Self {
        Self { secrets_repository }
    }
}

#[async_trait]
impl SecretsResolver for LocalSecretsResolver {
    async fn get_secret(&self, secret_id: &str) -> Result<Option<Secret>> {
        let secret = self
            .secrets_repository
            .clone()
            .find_one_by(doc! {"kid": secret_id})
            .await
            .map(|s| {
                s.map(|s| Secret {
                    id: s.kid,
                    type_: s.type_,
                    secret_material: s.secret_material,
                })
            })
            .map_err(|e| Error::new(ErrorKind::IoError, e))?;

        Ok(secret)
    }

    async fn find_secrets<'a>(&self, secret_ids: &'a [&'a str]) -> Result<Vec<&'a str>> {
        let mut found_secret_ids = Vec::new();

        for secret_id in secret_ids.iter() {
            if self
                .secrets_repository
                .clone()
                .find_one_by(doc! {"kid": *secret_id})
                .await
                .map_err(|e| Error::new(ErrorKind::IoError, e))?
                .is_some()
            {
                found_secret_ids.push(*secret_id);
            }
        }

        Ok(found_secret_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        repository::tests::MockSecretsRepository,
        utils::{self, filesystem::MockFileSystem},
    };
    use didcomm::secrets::{SecretMaterial, SecretType};
    use serde_json::{json, Value};

    fn setup() -> Document {
        let mock_fs = MockFileSystem;
        utils::read_diddoc(&mock_fs, "").unwrap()
    }

    #[tokio::test]
    async fn test_local_did_resolver_works() {
        let diddoc = setup();
        let resolver = LocalDIDResolver::new(&diddoc);

        let did = "did:web:alice-mediator.com:alice_mediator_pub";
        let resolved = resolver.resolve(did).await.unwrap().unwrap();
        let expected = serde_json::from_str::<Value>(
            r#"{
                "id": "did:web:alice-mediator.com:alice_mediator_pub",
                "verificationMethod": [
                    {
                        "id": "did:web:alice-mediator.com:alice_mediator_pub#keys-1",
                        "type": "JsonWebKey2020",
                        "controller": "did:web:alice-mediator.com:alice_mediator_pub",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4"
                        }
                    },
                    {
                        "id": "did:web:alice-mediator.com:alice_mediator_pub#keys-2",
                        "type": "JsonWebKey2020",
                        "controller": "did:web:alice-mediator.com:alice_mediator_pub",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4"
                        }
                    },
                    {
                        "id": "did:web:alice-mediator.com:alice_mediator_pub#keys-3",
                        "type": "JsonWebKey2020",
                        "controller": "did:web:alice-mediator.com:alice_mediator_pub",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "X25519",
                            "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ"
                        }
                    }
                ],
                "authentication": [
                    "did:web:alice-mediator.com:alice_mediator_pub#keys-1"
                ],
                "keyAgreement": [
                    "did:web:alice-mediator.com:alice_mediator_pub#keys-3"
                ],
                "service": []
            }"#,
        )
        .unwrap();

        assert_eq!(
            json_canon::to_string(&resolved).unwrap(),
            json_canon::to_string(&expected).unwrap()
        );

        let did = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7";
        let resolved = resolver.resolve(did).await.unwrap().unwrap();
        let expected = serde_json::from_str::<Value>(
            r#"{
                "id": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                "keyAgreement": [
                    "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr"
                ],
                "authentication": [
                    "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"
                ],
                "verificationMethod": [
                    {
                        "id": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                        "type": "JsonWebKey2020",
                        "controller": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                        "publicKeyJwk": {
                            "crv": "Ed25519",
                            "kty": "OKP",
                            "x": "Fpf4juyZWYUNmC8Bv87MmFLDWApxqOYYZUhWyiD7lSo"
                        }
                    },
                    {
                        "id": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr",
                        "type": "JsonWebKey2020",
                        "controller": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                        "publicKeyJwk": {
                            "crv": "X25519",
                            "kty": "OKP",
                            "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU"
                        }
                    }
                ],
                "service": []
            }"#,
        )
        .unwrap();

        assert_eq!(
            json_canon::to_string(&resolved).unwrap(),
            json_canon::to_string(&expected).unwrap()
        );
    }

    #[tokio::test]
    async fn test_local_did_resolver_fails_as_expected() {
        let diddoc = setup();
        let resolver = LocalDIDResolver::new(&diddoc);

        let did = "did:web:wrong-example.com";
        let resolved = resolver.resolve(did).await;
        assert!(matches!(
            resolved.unwrap_err().kind(),
            ErrorKind::Unsupported
        ));

        let did = "did:sov:wrong-example.com";
        let resolved = resolver.resolve(did).await;
        assert!(matches!(
            resolved.unwrap_err().kind(),
            ErrorKind::Unsupported
        ));

        let did = "did:key:Z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let resolved = resolver.resolve(did).await;
        assert!(matches!(
            resolved.unwrap_err().kind(),
            ErrorKind::DIDNotResolved
        ));
    }

    #[tokio::test]
    async fn test_local_secrets_resolver_works() {
        let secret_id = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr";
        let secret: Value = json!(
            {
                "kty": "OKP",
                "crv": "X25519",
                "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU",
                "d": "oItI6Jx-anGyhiDJIXtVAhzugOha05s-7_a5_CTs_V4"
            }
        );

        let test_secret = Secrets {
            id: None,
            kid: secret_id.to_string(),
            type_: SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: secret,
            },
        };

        let secrets_repository = Arc::new(MockSecretsRepository::from(vec![test_secret]));

        let resolver = LocalSecretsResolver::new(secrets_repository);
        let resolved = resolver.get_secret(secret_id).await.unwrap().unwrap();
        let expected = serde_json::from_str::<Value>(
            r#"{
                "id": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr",
                "type": "JsonWebKey2020",
                "privateKeyJwk": {
                    "crv": "X25519",
                    "d": "oItI6Jx-anGyhiDJIXtVAhzugOha05s-7_a5_CTs_V4",
                    "kty": "OKP",
                    "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            json_canon::to_string(&resolved).unwrap(),
            json_canon::to_string(&expected).unwrap()
        );

        let secret_id = "did:key:unregistered";
        let resolved = resolver.get_secret(secret_id).await.unwrap();
        assert!(resolved.is_none());
    }
}
