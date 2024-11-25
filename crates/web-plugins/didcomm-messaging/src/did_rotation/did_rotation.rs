use super::errors::RotationError;
use axum::response::{IntoResponse, Response};
use database::{Repository, RepositoryError};
use did_utils::didcore::Document as DidDocument;
use didcomm::{FromPrior, Message};
use mongodb::bson::doc;
use shared::{repository::entity::Connection, utils::resolvers::LocalDIDResolver};
use std::sync::Arc;

/// https://identity.foundation/didcomm-messaging/spec/#did-rotation
pub async fn did_rotation(
    msg: Message,
    connection_repos: &Arc<dyn Repository<Connection>>,
) -> Result<(), Response> {
    // Check if from_prior is none
    if msg.from_prior.is_none() {
        return Ok(());
    }
    let jwt = msg.from_prior.unwrap();
    let did_resolver = LocalDIDResolver::new(&DidDocument::default());

    // decode and validate jwt signature
    let (from_prior, _kid) = FromPrior::unpack(&jwt, &did_resolver)
        .await
        .map_err(|_| RotationError::InvalidFromPrior.json().into_response())?;
    let prev = from_prior.iss;

    // validate if did is  known
    let _ = match connection_repos
        .find_one_by(doc! {"client_did": &prev})
        .await
        .unwrap()
    {
        Some(mut connection) => {
            // get new did for communication, if empty then we end the relationship
            let new = from_prior.sub;

            if new.is_empty() {
                let id = connection.id.unwrap_or_default();
                return connection_repos
                    .delete_one(id)
                    .await
                    .map_err(|_| RotationError::TargetNotFound.json().into_response());
            }

            let did_index = connection.keylist.iter().position(|did| did == &prev);

            if did_index.is_some() {
                connection.keylist.swap_remove(did_index.unwrap());

                connection.keylist.push(new.clone());
            } else {
                // scenario in which there is rotation prior to keylist update
                connection.keylist.push(new.clone());
            }

            // store updated connection
            let _confirmations: Result<Connection, RepositoryError> = match connection_repos
                .update(Connection {
                    client_did: new,
                    ..connection
                })
                .await
            {
                Ok(conn) => Ok(conn),
                Err(_) => return Err(RotationError::RepositoryError.json().into_response()),
            };
        }

        None => {
            return Err(RotationError::UnknownIssuer.json().into_response())?;
        }
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, vec};
    use crate::constants::DIDCOMM_ENCRYPTED_MIME_TYPE;

    use did_utils::{didcore::Document, jwk::Jwk};
    use didcomm::secrets::SecretsResolver;
    use hyper::{header::CONTENT_TYPE, Body, Method, Request, StatusCode};
    use mongodb::bson::doc;
    use tower::ServiceExt;

    use keystore::{tests::MockKeyStore, Secrets};
    use shared::{
        repository::{
            entity::Connection,
            tests::{MockConnectionRepository, MockMessagesRepository},
        },
        state::{AppState, AppStateRepository},
        utils::resolvers::{LocalDIDResolver, LocalSecretsResolver},
    };

    pub fn new_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkqvgpxveKbuygKXnoRcD3jtLTJLgv7g6asLGLsoC4sUEp#z6LSeQmJnBaXhHz81dCGNDeTUUdMcX1a8p5YSVacaZEDdscp";
        let secret_material: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "d": "EIR1SxQ67uhVaeUd__sJZ_9pLLgtbVTq12Km8FI5TWY",
                "x": "KKBfakcXdzmJ3hhL0mVDg8OIwhTr9rPg_gvc-kPQpCU"
            }"#,
        )
        .unwrap();

        let secret = Secrets {
            id: None,
            kid: secret_id.to_string(),
            secret_material,
        };

        let keystore = MockKeyStore::new(vec![secret]);
        LocalSecretsResolver::new(Arc::new(keystore))
    }

    pub fn prev_did() -> String {
        "did:key:z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM".to_string()
    }
    pub fn new_did() -> String {
        "did:key:z6MkqvgpxveKbuygKXnoRcD3jtLTJLgv7g6asLGLsoC4sUEp".to_string()
    }
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let keys = vec![
            Secrets {
                id: None,
                kid: String::from("did:peer:2.Vz6Mkf6r1uMJwoRAbzkuyj2RwPusdZhWSPeEknnTcKv2C2EN7.Ez6LSgbP4b3y8HVWG6C73WF2zLbzjDAPXjc33P2VfnVVHE347.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-1"),
                secret_material: serde_json::from_str(
                    r#"{
                        "kty": "OKP",
                        "crv": "Ed25519",
                        "x": "CaDmpOjPAiMWfdzBcK2pLyJAER6xvdhDl2dro6BoilQ",
                        "d": "vp0WuZNeCsoXYj94738e0gwi_PLF7VIutNCrFVNx--0"
                    }"#,
                )
                .unwrap(),
            },
            Secrets {
                id: None,
                kid: String::from("did:peer:2.Vz6Mkf6r1uMJwoRAbzkuyj2RwPusdZhWSPeEknnTcKv2C2EN7.Ez6LSgbP4b3y8HVWG6C73WF2zLbzjDAPXjc33P2VfnVVHE347.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0#key-2"),
                secret_material: serde_json::from_str(
                    r#"{
                        "kty": "OKP",
                        "crv": "X25519",
                        "x": "SQ_7useLAjGf66XAwQWuBuSv9PdD_wB4TJQ6w38nFwQ",
                        "d": "kxUXT-2TOa6F6xk2ojQgJlT3xWq0aCA9j-BW4VB5_A8"
                    }"#,
                )
                .unwrap(),
            },
        ];

        let diddoc = didoc();
        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(_initial_connections())),
            keystore: Arc::new(MockKeyStore::new(keys)),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
        };

        let state = Arc::new(AppState::from(public_domain, diddoc, Some(repository)));

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
                    "mediator_did": "did:peer:2.Vz6Mkf6r1uMJwoRAbzkuyj2RwPusdZhWSPeEknnTcKv2C2EN7.Ez6LSgbP4b3y8HVWG6C73WF2zLbzjDAPXjc33P2VfnVVHE347.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                    "routing_did": "did:key:generated",
                    "keylist": [
                        "{_recipient_did}"
                        ]
                    }}
                    ]"##
        );

        serde_json::from_str(&connections).unwrap()
    }

    use didcomm::{FromPrior, Message};
    use serde_json::json;
    use uuid::Uuid;

    use super::did_rotation;

    fn didoc() -> Document {
        let doc: did_utils::didcore::Document = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/jws-2020/v1"
                ],
                "id": "did:peer:2.Vz6Mkf6r1uMJwoRAbzkuyj2RwPusdZhWSPeEknnTcKv2C2EN7.Ez6LSgbP4b3y8HVWG6C73WF2zLbzjDAPXjc33P2VfnVVHE347.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                "alsoKnownAs": [
                    "did:peer:3zQmNVZUh4qgAxSWhpeGhJVW3HHHU7MZZbZbQ4Vc43madsSf"
                ],
                "verificationMethod": [
                    {
                    "id": "#key-1",
                    "type": "JsonWebKey2020",
                    "controller": "did:peer:2.Vz6Mkf6r1uMJwoRAbzkuyj2RwPusdZhWSPeEknnTcKv2C2EN7.Ez6LSgbP4b3y8HVWG6C73WF2zLbzjDAPXjc33P2VfnVVHE347.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                    "publicKeyJwk": {
                        "kty": "OKP",
                        "crv": "Ed25519",
                        "x": "CaDmpOjPAiMWfdzBcK2pLyJAER6xvdhDl2dro6BoilQ"
                    }
                    },
                    {
                    "id": "#key-2",
                    "type": "JsonWebKey2020",
                    "controller": "did:peer:2.Vz6Mkf6r1uMJwoRAbzkuyj2RwPusdZhWSPeEknnTcKv2C2EN7.Ez6LSgbP4b3y8HVWG6C73WF2zLbzjDAPXjc33P2VfnVVHE347.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                    "publicKeyJwk": {
                        "kty": "OKP",
                        "crv": "X25519",
                        "x": "SQ_7useLAjGf66XAwQWuBuSv9PdD_wB4TJQ6w38nFwQ"
                    }
                    }
                ],
                "authentication": [
                    "#key-1"
                ],
                "keyAgreement": [
                    "#key-2"
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
            }"##,
        )
        .unwrap();
        doc
    }

    async fn test_jwt_data() -> String {
        pub fn prev_secrets_resolver() -> impl SecretsResolver {
            let secret_id = "did:key:z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM#z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM";
            let secret_material: Jwk = serde_json::from_str(
                r#"{
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "x": "sZPvulKOXCES3D8Eya3LVnlgOpEaBohCqZ7emD8VXAA",
                    "d": "kUKFMD3RCZpk556fG0hx9GUrmdvb8t7k3TktPXCi4CY"
                }"#,
            )
            .unwrap();

            let secret = Secrets {
                id: None,
                kid: secret_id.into(),
                secret_material,
            };

            let keystore = MockKeyStore::new(vec![secret]);
            LocalSecretsResolver::new(Arc::new(keystore))
        }

        let from_prior = FromPrior {
            iss: prev_did(),
            sub: new_did(),
            aud: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };

        let did_resolver = LocalDIDResolver::new(&didoc());
        let kid = "did:key:z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM#z6MkrQT3VKYGkbPaYuJeBv31gNgpmVtRWP5yTocLDBgPpayM";
        let (jwt, _kid) = from_prior
            .pack(Some(&kid), &did_resolver, &prev_secrets_resolver())
            .await
            .unwrap();
        jwt
    }

    fn test_message_payload(jwt: String) -> Message {
        let msg = Message::build(
            Uuid::new_v4().to_string(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
            json!({"updates": [
            {
                "recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                "action": "add"
            },
            {
                "recipient_did": "did:key:alice_identity_pub2@alice_mediator",
                "action": "remove"
            }
            ]}),
        )
        .header("return_route".into(), json!("all"))
        .to("did:peer:2.Vz6Mkf6r1uMJwoRAbzkuyj2RwPusdZhWSPeEknnTcKv2C2EN7.Ez6LSgbP4b3y8HVWG6C73WF2zLbzjDAPXjc33P2VfnVVHE347.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0".to_string())
        .from(new_did())
        .from_prior(jwt)
        .finalize();
        msg
    }

    #[tokio::test]
    async fn unit_test_on_did_rotation() {
        let jwt = test_jwt_data().await;
        let state = setup();
        let AppStateRepository {
            connection_repository,
            ..
        } = state.repository.as_ref().unwrap();

        let msg = test_message_payload(jwt);
        did_rotation(msg, &connection_repository).await.unwrap();

        // assert if did was rotated on mediator's site
        let _ = match connection_repository
            .find_one_by(doc! {"client_did": new_did()})
            .await
            .unwrap()
        {
            Some(conn) => {
                assert_eq!(conn.client_did, new_did())
            }
            None => {
                panic!("Rotation Error")
            }
        };
    }
}
