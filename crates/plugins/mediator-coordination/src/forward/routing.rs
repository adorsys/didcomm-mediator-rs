use didcomm::{Message, PackEncryptedOptions, UnpackOptions};

use crate::web::{error::MediationError, AppState};

pub async fn mediator_forward_process(
    payload: &str,
    state: &AppState,
    mut store: Vec<String>,
) -> Result<Vec<String>, MediationError> {
    // unpack encrypted payload message

    let result = Message::unpack(
        payload,
        &state.did_resolver,
        &state.secrets_resolver,
        &UnpackOptions::default(),
    )
    .await;
    {
        match result {
            Ok((unpack_msg, _)) => {
                if unpack_msg.to.is_some() {
                    let dids = Some(unpack_msg.clone().to).unwrap().unwrap();
                    for did in dids {
                        let (re_packed_msg, _) = unpack_msg
                            .pack_encrypted(
                                &did,
                                Some(unpack_msg.from.clone()).unwrap().as_deref(),
                                None,
                                &state.did_resolver,
                                &state.secrets_resolver,
                                &PackEncryptedOptions::default(),
                            )
                            .await
                            .unwrap();
                        store.push(re_packed_msg)
                    }
                }

                Ok(store)
            }
            Err(_) => Err(MediationError::MessageUnpackingFailure),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Borrow, sync::Arc};

    use crate::{
        repository::stateful::coord::tests::{MockConnectionRepository, MockSecretsRepository},
        util::{self, MockFileSystem},
        web::AppStateRepository,
    };

    use super::*;

    use did_utils::jwk::Jwk;
    use didcomm::{
        secrets::{Secret, SecretMaterial, SecretType},
        Message,
    };
    use serde_json::json;

    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        // generating secrets
        let jwk: Jwk = serde_json::from_str(
            r#"{
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "x": "tjOTPcs4OEMNrmn2ScYZDS-aCCbRFhJgaAmGnRsdmEo"
                }"#,
        )
        .unwrap();

        let secret = Secret {
                id: "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr".to_owned(),
                type_: SecretType::JsonWebKey2020,
                secret_material: SecretMaterial::JWK { private_key_jwk: json!(jwk) }
            };

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            secret_repository: Arc::new(MockSecretsRepository::from(vec![])),
        };

        let state = Arc::new(AppState::from(
            public_domain,
            diddoc,
            keystore,
            Some(repository),
        ));

        state
    }
    #[tokio::test]
    async fn test_mediator_forward_process() {
        let msg: Message = Message::build(
            "id".to_owned(),
            "type_".to_owned(),
            serde_json::json!("example-body"),
        )
        .to("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_owned())
        .from("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_owned())
        .finalize();
        let serialize_msg = serde_json::to_string(msg.clone().borrow());
        let state = setup();
        let store: Vec<String> = Vec::new();
        let pickup_msg = mediator_forward_process(serialize_msg.unwrap().as_str(), &state, store)
            .await
            .unwrap();
        for msg in pickup_msg {
            println!("{msg}")
        }
    }
}
