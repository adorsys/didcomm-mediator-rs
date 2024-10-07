use super::errors::RotationError;
use crate::{didcomm::bridge::LocalDIDResolver, model::stateful::entity::Connection};
use axum::response::{IntoResponse, Response};
use database::Repository;
use didcomm::{FromPrior, Message};
use mongodb::bson::doc;
use serde_json::Error;
use std::sync::Arc;

#[derive(Debug)]
pub enum Errors {
    Error0(RotationError),
    Error1(Error),
    Error2(Response),
}

pub async fn did_rotation(
    msg: Message,
    conection_repos: &Arc<dyn Repository<Connection>>,
) -> Result<(), Errors> {
    // Check if from_prior is not none
    if msg.from_prior.is_some() {
        let jwt = msg.from_prior.unwrap();
        let did_resolver = LocalDIDResolver::default();

        // decode and valid jwt signature
        let (from_prior, _kid) = FromPrior::unpack(&jwt, &did_resolver)
            .await
            .map_err(|_| Errors::Error2(RotationError::InvalidFromPrior.json().into_response()))?;

        let prev = from_prior.iss;

        // validate if did is  known
        let _connection = match conection_repos
            .find_one_by(doc! {"keylist": doc!{ "$elemMatch": { "$eq": &prev}}})
            .await
            .unwrap()
        {
            Some(mut connection) => {
                // stored the new did for communication
                let new = from_prior.sub;
                if connection.client_did == prev {
                    let _ = connection.client_did.replace(&prev, &new);
                };
                let did_index = connection
                    .keylist
                    .iter()
                    .position(|did| did == &prev)
                    .unwrap();
                connection.keylist.swap_remove(did_index).push_str(&new);
            }
            None => {
                return Err(Errors::Error0(RotationError::RotationError))?;
            }
        };
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use did_utils::jwk::Jwk;
    use didcomm::secrets::SecretsResolver;

    use crate::didcomm::bridge::LocalSecretsResolver;

    pub fn prev_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM#z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "sZPvulKOXCES3D8Eya3LVnlgOpEaBohCqZ7emD8VXAA",
                "d": "kUKFMD3RCZpk556fG0hx9GUrmdvb8t7k3TktPXCi4CY"
                }"#,
        )
        .unwrap();

        LocalSecretsResolver::new(&secret_id, &secret)
    }

    pub fn new_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkqvgpxveKbuygKXnoRcD3jtLTJLgv7g6asLGLsoC4sUEp#z6LSeQmJnBaXhHz81dCGNDeTUUdMcX1a8p5YSVacaZEDdscp";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "d": "EIR1SxQ67uhVaeUd__sJZ_9pLLgtbVTq12Km8FI5TWY",
                "x": "KKBfakcXdzmJ3hhL0mVDg8OIwhTr9rPg_gvc-kPQpCU"
            }"#,
        )
        .unwrap();
        LocalSecretsResolver::new(&secret_id, &secret)
    }
    pub fn prev_did() -> String {
        "did:key:z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM".to_string()
    }
    pub fn new_did() -> String {
        "did:key:z6MkqvgpxveKbuygKXnoRcD3jtLTJLgv7g6asLGLsoC4sUEp".to_string()
    }
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let storage_dirpath: String =
            std::env::var("STORAGE_DIRPATH").unwrap_or_else(|_| "/".to_owned());
        let diddoc: did_utils::didcore::Document =
            util::read_diddoc(&mock_fs, &storage_dirpath).unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(_initial_connections())),
            secret_repository: Arc::new(MockSecretsRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
        };

        let state = Arc::new(AppState::from(
            public_domain,
            diddoc,
            keystore,
            Some(repository),
        ));

        state
    }
    fn _initial_connections() -> Vec<Connection> {
        let _recipient_did = prev_did();

        let connections = format!(
            r##"[
            {{
            "_id": {{
                "$oid": "6580701fd2d92bb3cd291b2a"
                }},
                
                "client_did": "{_recipient_did}",
                "mediator_did": "did:web:alice-mediator.com:alice_mediator_pub",
                "routing_did": "did:key:generated",
                "keylist": [
                    "{_recipient_did}"
                    ]
                }}
                ]"##
        );

        serde_json::from_str(&connections).unwrap()
    }
    use std::sync::Arc;

    use didcomm::{FromPrior, Message};
    use serde_json::json;
    use uuid::Uuid;

    use crate::{
        didcomm::bridge::LocalDIDResolver,
        model::stateful::entity::Connection,
        repository::stateful::tests::{
            MockConnectionRepository, MockMessagesRepository, MockSecretsRepository,
        },
        rotation::rotation::did_rotation,
        util::{self, MockFileSystem},
        web::{AppState, AppStateRepository},
    };

    #[tokio::test]
    async fn test_did_rotation() {
        let doc: did_utils::didcore::Document = serde_json::from_str(
            r#"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/jws-2020/v1"
                ],
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
        let state = &setup();

        let from_prior = FromPrior {
            iss: prev_did(),
            sub: new_did(),
            aud: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };
        // let claims = serde_json::to_string(&from_prior).unwrap();
        let did_resolver = LocalDIDResolver::new(&doc);
        let kid = "did:key:z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM#z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM";
        let (jwt, _kid) = from_prior
            .pack(Some(&kid), &did_resolver, &prev_secrets_resolver())
            .await
            .unwrap();
        println!("{jwt}");

        let msg = Message::build(
            Uuid::new_v4().to_string(),
            "example/v1".to_owned(),
            json!(""),
        )
        .to("did:web:alice-mediator.com:alice_mediator_pub".to_string())
        .from(new_did())
        .from_prior(jwt)
        .finalize();
        let AppStateRepository {
            connection_repository,
            ..
        } = state.repository.as_ref().unwrap();
        println!("{:?}", did_rotation(msg, connection_repository).await);
        // let (msg, _) = msg
        //     .pack_encrypted(
        //         "did:web:alice-mediator.com:alice_mediator_pub",
        //         Some(&new_did()),
        //         None,
        //         &did_resolver,
        //         &new_secrets_resolver(),
        //         &didcomm::PackEncryptedOptions::default(),
        //     )
        //     .await
        //     .unwrap();

        // // Mediator in action
        // let secret: Jwk = serde_json::from_str(
        //     r#"{
        //         "kty": "OKP",
        //         "crv": "X25519",
        //         "d": "EIR1SxQ67uhVaeUd__sJZ_9pLLgtbVTq12Km8FI5TWY",
        //         "x": "KKBfakcXdzmJ3hhL0mVDg8OIwhTr9rPg_gvc-kPQpCU"
        //         }"#,
        //     )
        //     .unwrap();
        // let did_resolver = LocalDIDResolver::new(&doc);
        // let secrets_resolver = LocalSecretsResolver::new("did:key:z6MkqvgpxveKbuygKXnoRcD3jtLTJLgv7g6asLGLsoC4sUEp#z6LSeQmJnBaXhHz81dCGNDeTUUdMcX1a8p5YSVacaZEDdscp", &secret);

        // let msg = Message::unpack(
        //     &msg,
        //     &state.did_resolver,
        //     &secret_resolver,
        //     &didcomm::UnpackOptions::default(),
        // )
        // .await
        // .unwrap();
    }
}
