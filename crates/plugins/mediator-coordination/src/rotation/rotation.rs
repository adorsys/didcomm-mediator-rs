use std::collections::BTreeMap;
use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use database::Repository;
use didcomm::error::Error as Didcommerr;
use didcomm::{did::DIDResolver, FromPrior, Message};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use mongodb::bson::doc;
use serde_json::Error;
use sha2::Sha256;

use crate::{didcomm::bridge::LocalDIDResolver, model::stateful::entity::Connection};

use super::errors::RotationError;
pub enum Errors {
    Error0(RotationError),
    Error1(Error),
}

pub async fn did_rotation(
    msg: Message,
    conection_repos: &Arc<dyn Repository<Connection>>,
) -> Result<(), Errors> {
    // Check if from_prior is not none
    if msg.from_prior.is_some() {
        let jwt = msg.from_prior.unwrap();
        let did_resolver = LocalDIDResolver::default();
        let (from_prior, kid) = FromPrior::unpack(&jwt, &did_resolver).await.unwrap(); // todo find different way to handle

        let prev = from_prior.iss;

        // validate if did is  known
        let _connection = match conection_repos
            .find_one_by(doc! {"client_did": &prev})
            .await
            .unwrap()
        {
            Some(connection) => {
                // validate jwt signatures with previous did kid
                let key: Hmac<Sha256> = Hmac::new_from_slice(kid.as_bytes()).unwrap(); // todo find different way to handle
                let _: BTreeMap<String, String> = jwt.verify_with_key(&key).unwrap(); // todo find different way to handle

                // stored the new did for communication
                let new = from_prior.sub;
                connection.client_did.replace(&prev, &new)
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
    pub fn _recipient_did() -> String {
        "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
    }
    pub fn _sender_did() -> String {
        "did:key:z6MkwKfDFAK49Lb9D6HchFiCXdcurRUSFrbnwDBk5qFZeHA3".to_string()
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
        let _recipient_did = _recipient_did();

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

    use didcomm::{
        did::{self, DIDResolver}, secrets::resolvers::ExampleSecretsResolver, FromPrior, Message, PackEncryptedOptions, UnpackOptions
    };
    use jwt::ToBase64;
    use serde_json::json;
    use uuid::Uuid;

    use crate::{
        didcomm::bridge::{LocalDIDResolver, LocalSecretsResolver},
        model::stateful::entity::Connection,
        repository::stateful::tests::{
            MockConnectionRepository, MockMessagesRepository, MockSecretsRepository,
        },
        util::{self, MockFileSystem},
        web::{AppState, AppStateRepository},
    };

    #[tokio::test]
    async fn test_did_rotation() {
        let state = &setup();

        let from_prior = FromPrior {
            iss: _sender_did(),
            sub: "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH5".to_string(),
            aud: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };
        let claims = serde_json::to_string(&from_prior).unwrap();

        let diddoc = LocalDIDResolver::resolve(&LocalDIDResolver::default(), &_sender_did())
            .await
            .unwrap()
            .unwrap();
        let kid = diddoc.verification_method.get(0).unwrap().id;

        let key = jsonwebtoken::EncodingKey::from_secret(&kid.as_bytes());

        // encoding from_prior to jwt
        let header = jsonwebtoken::Header::default();
        let jwt = jsonwebtoken::encode(&header, &claims, &key).unwrap();

        let msg = Message::build(
            Uuid::new_v4().to_string(),
            "example/v1".to_owned(),
            json!(""),
        )
        .to(_recipient_did())
        .from(_sender_did())
        .from_prior(jwt)
        .finalize();
        let msg = msg
            .pack_encrypted(
               "did:web:alice-mediator.com:alice_mediator_pub",
                Some(&_sender_did()),
                None,
                &state.did_resolver,
                &state.secrets_resolver,
                &PackEncryptedOptions::default(),
            )
            .await
            .unwrap();

        // Mediator in action
       let  did_resolver = LocalDIDResolver::default();
       
        let msg = Message::unpack(msg.0, &did_resolver, secrets_resolver, options).await.unwrap();
        did_rotation(msg, conection_repos)
    }
}
