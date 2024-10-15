#[cfg(test)]
pub mod tests {
    use crate::{
        repository::{
            entity::Secrets,
            tests::{MockConnectionRepository, MockMessagesRepository, MockSecretsRepository},
        },
        resolvers::LocalSecretsResolver,
        state::{AppState, AppStateRepository},
        util::{self, MockFileSystem},
    };
    use didcomm::{
        error::Error as DidcommError,
        secrets::{SecretMaterial, SecretType, SecretsResolver},
        Message, PackEncryptedOptions, UnpackOptions,
    };
    use serde_json::{json, Value};
    use std::sync::Arc;

    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mock_fs = MockFileSystem;
        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            secret_repository: Arc::new(MockSecretsRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
        };

        let state = Arc::new(AppState::from(public_domain, diddoc, Some(repository)));

        state
    }

    pub fn _mediator_did(state: &AppState) -> String {
        state.diddoc.id.clone()
    }

    pub fn _edge_did() -> String {
        "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
    }

    pub fn _edge_signing_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7";
        let secret: Value = json!(
            {
                "kty": "OKP",
                "crv": "Ed25519",
                "d": "UXBdR4u4bnHHEaDK-dqE04DIMvegx9_ZOjm--eGqHiI",
                "x": "Fpf4juyZWYUNmC8Bv87MmFLDWApxqOYYZUhWyiD7lSo",
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

        LocalSecretsResolver::new(secrets_repository)
    }

    pub fn _edge_secrets_resolver() -> impl SecretsResolver {
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

        LocalSecretsResolver::new(secrets_repository)
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
