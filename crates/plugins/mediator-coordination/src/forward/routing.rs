use std::clone;

use axum::response::{IntoResponse, Response};
use didcomm::Message;

use hyper::StatusCode;
use mongodb::bson::doc;
use serde_json::{from_value, json, Value};

use crate::{
    model::stateful::{coord::KeylistEntry, entity::Messages},
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

    let body: Value = json!(payload.body.as_object());
    let next: Vec<String> = from_value(body.get("next").unwrap().to_owned()).unwrap();

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
            let message = payload.clone().attachments.expect("expect attachements");
            let receivering_dids = next;
            for did in receivering_dids {
                let messages = Messages {
                    id: None,
                    message: message.clone(),
                    recipient_did: did,
                };
                message_repository
                    .store(messages)
                    .await
                    .map_err(|_| MediationError::PersisenceError)
                    .unwrap();
            }
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
        constant::MEDIATE_FORWARD_2_0,
        forward::ledger::{ALICE_DID_DOC, ALICE_SECRETS, MEDIATOR_DID_DOC},
        repository::stateful::coord::tests::{
            MockConnectionRepository, MockMessagesRepository, MockSecretsRepository,
        },
        util::{self, MockFileSystem},
        web::AppStateRepository,
    };

    use super::*;

    use didcomm::{
        did::resolvers::ExampleDIDResolver, secrets::resolvers::ExampleSecretsResolver, Message,
        PackEncryptedOptions, UnpackOptions,
    };
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
        let did_resolver =
            ExampleDIDResolver::new(vec![MEDIATOR_DID_DOC.clone(), ALICE_DID_DOC.clone()]);
        let secret_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());
        const ALICE_DID: &str = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7";
        const MEDIATOR_DID: &str = "did:web:alice-mediator.com:alice_mediator_pub";
        let id = Uuid::new_v4().to_string();
        let msg: Message = Message::build(
            id,
            MEDIATE_FORWARD_2_0.to_string(),
            serde_json::json!({"next":["did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"]}),
        )
        .to(MEDIATOR_DID.to_owned())
        .from(ALICE_DID.to_owned())
        .finalize();
        let state = &setup();

        let (msg, _metadata) = msg
            .pack_encrypted(
                MEDIATOR_DID,
                Some(ALICE_DID),
                None,
                &did_resolver,
                &secret_resolver,
                &PackEncryptedOptions::default(),
            )
            .await
            .unwrap();

        // Mediator in action
        let (payload, _) = Message::unpack(
            &msg,
            &state.did_resolver,
            &state.secrets_resolver,
            &UnpackOptions::default(),
        )
        .await
        .unwrap();
        mediator_forward_process(state, payload).await.ok();
    }
}
