#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use crate::{
        breaker::CircuitBreaker,
        repository::tests::{MockConnectionRepository, MockMessagesRepository},
        state::{AppState, AppStateRepository},
        utils::resolvers::LocalSecretsResolver,
    };
    use did_utils::{didcore::Document, jwk::Jwk};
    use didcomm::{
        error::Error as DidcommError, secrets::SecretsResolver, Message, PackEncryptedOptions,
        UnpackOptions,
    };
    use keystore::Keystore;
    use std::{env, sync::Arc};

    pub fn setup() -> Arc<AppState> {
        env::set_var("MASTER_KEY", "1234567890qwertyuiopasdfghjklxzc");
        let public_domain = String::from("http://alice-mediator.com");

        let diddoc: Document = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/jws-2020/v1"
                ],
                "id": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                "alsoKnownAs": [
                    "did:peer:3zQmZo9aYaBjv2XtjRcTfP7X7QwyU1VVnrcEWVtcBhiAtPFa"
                ],
                "verificationMethod": [
                    {
                        "id": "#key-1",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "X25519",
                            "x": "_EgIPSRgbPPw5-nUsJ6xqMvw5rXn3BViGADeUrjAMzA"
                        }
                    },
                    {
                        "id": "#key-2",
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
                            "uri": "http://alice-mediator.com/"
                        }
                    }
                ]
            }"##
        ).unwrap();

        let secret_id = "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-1";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "_EgIPSRgbPPw5-nUsJ6xqMvw5rXn3BViGADeUrjAMzA",
                "d": "3S3CDZD0vqYN4fnxVratwv2Zq-LtIUgkNqUufR9udLQ"
            }"#,
        )
        .unwrap();

        let mediator_secret = (secret_id.to_string(), secret);
        let keystore = Keystore::with_mock_configs(vec![mediator_secret]);

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
            keystore,
        };

        Arc::new(
            AppState::from(
                public_domain,
                diddoc,
                None,
                Some(repository),
                CircuitBreaker::new(),
            )
            .unwrap(),
        )
    }

    pub fn _mediator_did(state: &AppState) -> String {
        state.diddoc.id.clone()
    }

    pub fn _edge_did() -> String {
        "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
    }

    pub fn _edge_signing_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "d": "UXBdR4u4bnHHEaDK-dqE04DIMvegx9_ZOjm--eGqHiI",
                "x": "Fpf4juyZWYUNmC8Bv87MmFLDWApxqOYYZUhWyiD7lSo",
            }"#,
        )
        .unwrap();

        let test_secret = (secret_id.to_string(), secret);
        let keystore = Keystore::with_mock_configs(vec![test_secret]);

        LocalSecretsResolver::new(keystore)
    }

    pub fn _edge_secrets_resolver() -> impl SecretsResolver {
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

        let test_secret = (secret_id.to_string(), secret);
        let keystore = Keystore::with_mock_configs(vec![test_secret]);

        LocalSecretsResolver::new(keystore)
    }

    pub async fn _edge_pack_message(
        state: &AppState,
        msg: &Message,
        from: Option<String>,
        to: String,
    ) -> Result<String, DidcommError> {
        let (packed, _) = msg
            .pack_encrypted(
                &to,
                from.as_deref(),
                None,
                &state.did_resolver,
                &_edge_secrets_resolver(),
                &PackEncryptedOptions::default(),
            )
            .await?;

        Ok(packed)
    }

    pub async fn _edge_unpack_message(
        state: &AppState,
        msg: &str,
    ) -> Result<Message, DidcommError> {
        let (unpacked, _) = Message::unpack(
            msg,
            &state.did_resolver,
            &_edge_secrets_resolver(),
            &UnpackOptions::default(),
        )
        .await
        .expect("Unable to unpack");

        Ok(unpacked)
    }
}
