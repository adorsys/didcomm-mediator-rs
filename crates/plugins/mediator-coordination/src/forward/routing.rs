

use axum::response::{IntoResponse, Response};
use didcomm::Message;

use hyper::StatusCode;
use mongodb::bson::doc;
use serde_json::{from_value, json, Value};

use crate::{
    model::stateful::entity::{Connection, RoutedMessage},
    web::{error::MediationError, AppState, AppStateRepository},
};

/// mediator receives messages of type forward then it unpacks the messages and stores it for pickup
/// the unpacked message is then repacked for further transmission.
/// Note: Stored messages are not re_packed and must be before transmission in case of
/// Rewrapping.
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
    let sender = payload.clone().from.unwrap();
    let _connection: Option<Connection> = match connection_repository
        .find_one_by(doc! {"client_did": &sender})
        .await
        .unwrap()
    {
        Some(_connection) => None,
        None => {
            let response = (
                StatusCode::UNAUTHORIZED,
                MediationError::UncoordinatedSender.json(),
            );
            return Err(response.into_response());

        }
    };

    // store unpacked payload with associated dids in the next field of body for routing
    let receivering_dids = next;
    for did in receivering_dids {
        let messages = RoutedMessage {
            id: None,
            message: payload.clone(),
            recipient_did: did,
        };
        message_repository
            .store(messages)
            .await
            .map_err(|_| MediationError::PersisenceError)
            .unwrap();
    }


 Ok(None)

}

#[cfg(test)]
mod test {
    use std::{borrow::Borrow, sync::Arc};

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
        did::resolvers::ExampleDIDResolver, secrets::resolvers::ExampleSecretsResolver, Attachment, AttachmentData, JsonAttachmentData, Message, PackEncryptedOptions, UnpackOptions
    };
    use uuid::Uuid;
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let storage_dirpath = std::env::var("STORAGE_DIRPATH").unwrap_or_else(|_| "/".to_owned());
        let diddoc = util::read_diddoc(&mock_fs,&storage_dirpath).unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            secret_repository: Arc::new(MockSecretsRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![]))
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

        let plaintext_msg = Attachment {
            id: None,
            description: Some("A friendly reminder to take a break and enjoy some fresh air!".to_string()),
            media_type: None,
            data: AttachmentData::Json { value: JsonAttachmentData{json: json!("Hey there! Just wanted to remind you to step outside for a bit. A little fresh air can do wonders for your mood."), jws: None} },
            filename: Some("reminder.txt".to_string()),
            format: Some("mime_type".to_string()),
            lastmod_time: None,
            byte_count: None
        };
        
        let forward_msg: Message = Message::build(

            id,
            MEDIATE_FORWARD_2_0.to_string(),
            serde_json::json!({"next":["did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"]}),
        )
        .to(MEDIATOR_DID.to_owned())
        .from(ALICE_DID.to_owned())
        .attachment(plaintext_msg)

        .finalize();
        let state = &setup();

        let (packed_forward_msg, _metadata) = forward_msg

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
            &packed_forward_msg,
            &state.did_resolver,
            &state.secrets_resolver,
            &UnpackOptions::default(),
        )
        .await
        .unwrap();
        mediator_forward_process(state, payload).await.ok();

    }
}
