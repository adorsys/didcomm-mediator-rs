
use axum::response::{IntoResponse, Response};
use didcomm::{protocols::routing::try_parse_forward, Message};

use hyper::StatusCode;
use mongodb::bson::doc;


use crate::{
    model::stateful::entity::Messages,
    web::{error::MediationError, AppState, AppStateRepository},
};

/// mediator receives messages of type forward then it unpacks the messages and stores it for pickup
/// the unpacked message is then repacked for further transmission.
pub async fn mediator_forward_process(
    state: &AppState,
    payload: Message,
) -> Result<Option<Message>, Response> {
    let AppStateRepository {
        message_repository,
        connection_repository,
        ..
    } = state
        .repository
        .as_ref()
        .ok_or_else(|| MediationError::RepostitoryError)
        .unwrap();

    // Check if the sender has a connection with the mediator else return early with custom error.
    let sender_did = payload.clone().from.unwrap();
    let connection = match connection_repository
        .find_one_by(doc! {"client_did": &sender_did})
        .await
        .unwrap()
    {
        Some(connection) => connection,
        None => {
            let response = (
                StatusCode::UNAUTHORIZED,
                MediationError::UncoordinatedSender.json(),
            );
            return Err(response.into_response());
        }
    };

    // check if sender's did in mediator's keylist
    let keylist_entries = connection.keylist.iter().find(|keys| keys == &&sender_did);
    match keylist_entries {
        Some(_) => {
            // store message attachement with associated recipient did
            let result = try_parse_forward(&payload).expect("Could Not Parse Forward");

            let forward_msg = serde_json::to_string(&result.forwarded_msg).unwrap();

            let messages = Messages {
                id: None,
                message: vec![forward_msg],
                recipient_did: result.next,
            };
            message_repository
                .store(messages)
                .await
                .map_err(|_| MediationError::PersisenceError)
                .unwrap();
        }
        None => {
            let response = (
                StatusCode::UNAUTHORIZED,
                MediationError::UncoordinatedSender.json(),
            );
            return Err(response.into_response());
        }
    }

    Ok(None)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{
        didcomm::bridge::LocalSecretsResolver,
        repository::stateful::coord::tests::{
            MockConnectionRepository, MockMessagesRepository, MockSecretsRepository,
        },
        util::{self, MockFileSystem},
        web::AppStateRepository,
    };

    use super::*;

    use did_utils::jwk::Jwk;
    use didcomm::{
        secrets::SecretsResolver,
        Message, PackEncryptedOptions, UnpackOptions,
    };
    use serde_json::json;
    use uuid::Uuid;
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
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

    #[tokio::test]
    async fn test_mediator_forward_process() {
        // simulate sender forwarding process
   let state = &setup();
        let msg = Message::build(
            Uuid::new_v4().to_string(),
            "example/v1".to_owned(),
            json!("Hey there! Just wanted to remind you to step outside for a bit. A little fresh air can do wonders for your mood."),
        )
        .to(_recipient_did())
        .from(_sender_did())
        .finalize();

        let (msg, _) = msg
            .pack_encrypted(
                &_recipient_did(),
                Some(&_sender_did()),
                None,
                &state.did_resolver,
                &_sender_secrets_resolver(),
                &PackEncryptedOptions::default(),
            )
            .await
            .expect("Unable pack_encrypted");

        // Mediator in action
        let (payload, _) = Message::unpack(
            &msg,
            &state.did_resolver,
            &state.secrets_resolver,
            &UnpackOptions::default(),
        )
        .await
        .unwrap();

        assert!(mediator_forward_process(state, payload).await.is_ok());
    }

    pub fn _sender_did() -> String {
        "did:key:z6MkwKfDFAK49Lb9D6HchFiCXdcurRUSFrbnwDBk5qFZeHA3".to_string()
    }

    pub fn _mediator_did(state: &AppState) -> String {
        state.diddoc.id.clone()
    }

    pub fn _recipient_did() -> String {
        "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
    }

    pub fn _sender_secrets_resolver() -> impl SecretsResolver {
        let secret_id = _sender_did() + "#z6LSiZbfm5L5zR3mrqpHyL7T2b2x3afUMpmGnMrEQznAz5F3";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "ZlJzHqy2dLrDQNlV15O3zDOIXpWVQnq6VtiVZ78O0hY",
                "d": "8OK7-1IVMdcM86PZzYKsbIi3kCJ-RxI8XFKe9JEcF2Y"
            }"#,
        )
        .unwrap();

        LocalSecretsResolver::new(&secret_id, &secret)
    }
}
