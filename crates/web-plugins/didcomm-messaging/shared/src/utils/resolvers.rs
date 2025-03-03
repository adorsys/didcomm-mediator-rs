use async_trait::async_trait;
use did_utils::{
    crypto::PublicKeyFormat,
    didcore::{Document, VerificationMethodType},
    jwk::Jwk,
    methods::{DidKey, DidPeer},
};
use didcomm::{
    did::{DIDDoc, DIDResolver},
    error::{Error, ErrorKind, Result},
    secrets::{Secret, SecretMaterial, SecretType, SecretsResolver},
};
use keystore::Keystore;
use serde_json::json;
use std::collections::HashSet;

#[derive(Clone)]
pub struct LocalDIDResolver {
    diddoc: Document,
}

impl LocalDIDResolver {
    pub fn new(server_diddoc: &Document) -> Self {
        Self {
            diddoc: serde_json::from_value(json!(server_diddoc)).unwrap_or_default(),
        }
    }
}

#[async_trait]
impl DIDResolver for LocalDIDResolver {
    async fn resolve(&self, did: &str) -> Result<Option<DIDDoc>> {
        if did == self.diddoc.id {
            let mut diddoc = self.diddoc.clone();
            prepend_doc_id_to_vm_ids(&mut diddoc);
            return Ok(Some(serde_json::from_value(json!(diddoc))?));
        }

        if did.starts_with("did:key") {
            let diddoc = DidKey::new_full(true, PublicKeyFormat::Jwk)
                .expand(did)
                .map_err(|e| Error::new(ErrorKind::DIDNotResolved, e))?;
            let diddoc = serde_json::from_value(json!(Document {
                service: Some(vec![]),
                ..diddoc
            }))?;
            Ok(Some(diddoc))
        } else if did.starts_with("did:peer") {
            let mut diddoc = DidPeer::with_format(PublicKeyFormat::Jwk)
                .expand(did)
                .map_err(|e| Error::new(ErrorKind::DIDNotResolved, e))?;
            prepend_doc_id_to_vm_ids(&mut diddoc);
            let diddoc = serde_json::from_value(json!(diddoc))?;
            Ok(Some(diddoc))
        } else {
            Err(Error::msg(
                ErrorKind::Unsupported,
                "Unsupported DID".to_string(),
            ))
        }
    }
}

fn prepend_doc_id_to_vm_ids(diddoc: &mut Document) {
    if let Some(verification_methods) = diddoc.verification_method.as_mut() {
        for vm in verification_methods.iter_mut() {
            vm.id = diddoc.id.to_owned() + &vm.id;
        }
    }

    let rel_prepend = |rel: &mut Option<Vec<VerificationMethodType>>| {
        if let Some(rel) = rel {
            for vm in rel.iter_mut() {
                if let VerificationMethodType::Reference(ref mut id) = vm {
                    *id = diddoc.id.to_owned() + id;
                }
            }
        }
    };

    rel_prepend(&mut diddoc.authentication);
    rel_prepend(&mut diddoc.key_agreement);
}

#[derive(Clone)]
pub struct LocalSecretsResolver {
    keystore: Keystore,
}

impl LocalSecretsResolver {
    pub fn new(keystore: Keystore) -> Self {
        Self { keystore }
    }
}
#[async_trait]
impl SecretsResolver for LocalSecretsResolver {
    async fn get_secret(&self, secret_id: &str) -> Result<Option<Secret>> {
        let secret = self
            .keystore
            .clone()
            .retrieve::<Jwk>(secret_id)
            .await
            .map(|s| {
                s.map(|s| Secret {
                    id: secret_id.to_string(),
                    type_: SecretType::JsonWebKey2020,
                    secret_material: SecretMaterial::JWK {
                        private_key_jwk: json!(s),
                    },
                })
            })
            .map_err(|e| Error::new(ErrorKind::IoError, e))?;

        Ok(secret)
    }

    async fn find_secrets<'a>(&self, secret_ids: &'a [&'a str]) -> Result<Vec<&'a str>> {
        let mut found_secret_ids = HashSet::with_capacity(secret_ids.len());

        for secret_id in secret_ids.iter() {
            if self
                .keystore
                .clone()
                .retrieve::<Jwk>(secret_id)
                .await
                .map_err(|e| Error::new(ErrorKind::IoError, e))?
                .is_some()
            {
                found_secret_ids.insert(*secret_id);
            }
        }

        Ok(found_secret_ids.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::tests_utils::tests;

    use super::*;
    use did_utils::jwk::Jwk;
    use serde_json::Value;

    fn setup() -> Document {
        tests::setup().clone().diddoc.clone()
    }

    #[tokio::test]
    async fn test_local_did_resolver_works() {
        let diddoc = setup();
        let resolver = LocalDIDResolver::new(&diddoc);

        let did = "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0";
        let resolved = resolver.resolve(did).await.unwrap().unwrap();
        let expected = serde_json::from_str::<Value>(
            r##"{
                "id": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                "verificationMethod": [
                    {
                        "id": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-1",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "X25519",
                            "x": "_EgIPSRgbPPw5-nUsJ6xqMvw5rXn3BViGADeUrjAMzA"
                        }
                    },
                    {
                        "id": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-2",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": "PuG2L5um-tAnHlvT29gTm9Wj9fZca16vfBCPKsHB5cA"
                        }
                    }
                ],
                "authentication": [
                    "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-2"
                ],
                "keyAgreement": [
                    "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-1"
                ],
                "service": [
                    {
                        "id": "#didcomm",
                        "type": "DIDCommMessaging",
                        "serviceEndpoint": {
                            "accept": [
                                "didcomm/v2"
                            ],
                            "routingKeys": [],
                            "uri": "http://alice-mediator.com/"
                        }
                    }
                ]
            }"##
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
        std::env::set_var("MASTER_KEY", "1234567890qwertyuiopasdfghjklxzc");
        let secret_id = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU",
                "d": "oItI6Jx-anGyhiDJIXtVAhzugOha05s-7_a5_CTs_V4"
            }"#,
        )
        .unwrap();

        let keystore = Keystore::with_mock_configs(vec![(secret_id.to_string(), secret)]);

        let resolver = LocalSecretsResolver::new(keystore);
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

    #[test]
    fn test_prepend_doc_id_to_vm_ids_works() {
        let mut diddoc: Document = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/jws-2020/v1"
                ],
                "id": "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                "alsoKnownAs": [
                    "did:peer:3zQmSBPjNZR15mNMUBKpTqk8Z4icxkv91zAG5GsnsGqZj6yY"
                ],
                "verificationMethod": [
                    {
                        "id": "#key-1",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "X25519",
                            "x": "AEtUMFyAEQte9YlqvsqiKK9uD_PFe1lXNZ_CiMRpahA"
                        }
                    },
                    {
                        "id": "#key-2",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": "1wfj-I-3zHB86RPIje5i6_jb0TeC67KF_mz8kdcyYqE"
                        }
                    }
                ],
                "authentication": [
                    "#key-2"
                ],
                "keyAgreement": [
                    "#key-1"
                ],
                "service": [
                    {
                        "id": "#didcomm",
                        "type": "DIDCommMessaging",
                        "serviceEndpoint": {
                            "accept": [
                                "didcomm/v2"
                            ],
                            "routingKeys": [],
                            "uri": "http://alice-mediator.com"
                        }
                    }
                ]
            }"##
        ).unwrap();

        prepend_doc_id_to_vm_ids(&mut diddoc);

        let expected = serde_json::from_str::<Value>(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/jws-2020/v1"
                ],
                "id": "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                "alsoKnownAs": [
                    "did:peer:3zQmSBPjNZR15mNMUBKpTqk8Z4icxkv91zAG5GsnsGqZj6yY"
                ],
                "verificationMethod": [
                    {
                        "id": "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-1",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "X25519",
                            "x": "AEtUMFyAEQte9YlqvsqiKK9uD_PFe1lXNZ_CiMRpahA"
                        }
                    },
                    {
                        "id": "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-2",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": "1wfj-I-3zHB86RPIje5i6_jb0TeC67KF_mz8kdcyYqE"
                        }
                    }
                ],
                "authentication": [
                    "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-2"
                ],
                "keyAgreement": [
                    "did:peer:2.Ez6LSbhKnZ7tsrvScZBR5mRSnVDa7S7km1aCpkHoWS1pkLhkj.Vz6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.Az6MktvegL6Tx3fPrNhhYbtxmzq6nsjnQKoecKLARJVZ7catQ.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-1"
                ],
                "service": [
                    {
                        "id": "#didcomm",
                        "type": "DIDCommMessaging",
                        "serviceEndpoint": {
                            "accept": [
                                "didcomm/v2"
                            ],
                            "routingKeys": [],
                            "uri": "http://alice-mediator.com"
                        }
                    }
                ]
            }"##
        ).unwrap();

        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),
            json_canon::to_string(&expected).unwrap()
        );
    }
}
