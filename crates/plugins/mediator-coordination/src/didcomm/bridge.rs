use async_trait::async_trait;
use did_utils::crypto::PublicKeyFormat;
use did_utils::methods::DidPeer;
use did_utils::{didcore::Document, jwk::Jwk, methods::DidKey};
use didcomm::{
    did::{DIDDoc, DIDResolver},
    error::{Error, ErrorKind, Result},
    secrets::{Secret, SecretMaterial, SecretType, SecretsResolver},
};
use serde_json::json;

#[derive(Clone)]
pub struct LocalDIDResolver {
    diddoc: DIDDoc,
}

impl LocalDIDResolver {
    pub fn new(server_diddoc: &Document) -> Self {
        Self {
            diddoc: server_diddoc.to_owned().into(),
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
                .map(|d| Some(d.into()))
                .map_err(|e| Error::new(ErrorKind::DIDNotResolved, e))?)
        } else if did.starts_with("did:peer") {
            Ok(DidPeer::new_with_format(PublicKeyFormat::Jwk)
                .expand(did)
                .map(|d| Some(d.into()))
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
    secrets: Vec<Secret>,
}

impl LocalSecretsResolver {
    pub fn new(secret_id: &str, secret: &Jwk) -> Self {
        Self {
            secrets: vec![Secret {
                id: secret_id.to_string(),
                type_: SecretType::JsonWebKey2020,
                secret_material: SecretMaterial::JWK {
                    private_key_jwk: json!(secret),
                },
            }],
        }
    }
}

#[async_trait]
impl SecretsResolver for LocalSecretsResolver {
    async fn get_secret(&self, secret_id: &str) -> Result<Option<Secret>> {
        Ok(self.secrets.iter().find(|s| s.id == secret_id).cloned())
    }

    async fn find_secrets<'a>(&self, secret_ids: &'a [&'a str]) -> Result<Vec<&'a str>> {
        Ok(secret_ids
            .iter()
            .filter(|&&sid| self.secrets.iter().any(|s| s.id == sid))
            .copied()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::Value;

    use crate::util::{self, MockFileSystem};

    fn setup() -> Document {
        let mock_fs = MockFileSystem;
        util::read_diddoc(&mock_fs, "").unwrap()
    }

    #[tokio::test]
    async fn test_local_did_resolver_works() {
        let _doc = LocalDIDResolver::new(&Document::default());
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
        let jwk: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU",
                "d": "oItI6Jx-anGyhiDJIXtVAhzugOha05s-7_a5_CTs_V4"
            }"#,
        )
        .unwrap();

        let resolver = LocalSecretsResolver::new(secret_id, &jwk);
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

