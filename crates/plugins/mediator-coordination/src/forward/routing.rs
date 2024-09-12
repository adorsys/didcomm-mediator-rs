use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use didcomm::{Message, UnpackOptions};
use hyper::StatusCode;
use serde::Deserialize;

use crate::{
    model::stateful::entity::Messages,
    web::{error::MediationError, AppState, AppStateRepository},
};
#[derive(Deserialize)]
struct Body {
    next: Vec<String>,
}
/// mediator receives messages of type forward then it unpacks the messages and stores it for pickup
/// the unpacked message is then repacked for further transmission.
/// Note: Stored messages are not re_packed and must be before transmission in case of
/// Rewrapping.
pub async fn mediator_forward_process(
    state: Arc<AppState>,
    payload: String,
) -> Result<Response, MediationError> {
    let AppStateRepository {
        message_repository, ..
    } = state
        .repository
        .as_ref()
        .ok_or_else(|| MediationError::RepostitoryError)?;

    // unpack encrypted payload message
    let (unpack_msg, _) = Message::unpack(
        payload.as_str(),
        &state.did_resolver,
        &state.secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .map_err(|_| MediationError::MessageUnpackingFailure)?;
    let body: Body = serde_json::from_str(&unpack_msg.body.as_str().unwrap()).unwrap();

    // store unpacked payload with associated dids in the next field of body for routing
    let receiver_dids = body.next;
    for did in receiver_dids {
        let messages = Messages {
            id: None,
            message: unpack_msg.clone(),
            recipient_did: did,
        };
        message_repository
            .store(messages)
            .await
            .map_err(|_| MediationError::PersisenceError)?;
    }

    Ok(StatusCode::OK.into_response())
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
        PackEncryptedOptions,
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
    /// simulate sender forwarding process

    async fn test_mediator_forward_process() {
        let did_resolver =
            ExampleDIDResolver::new(vec![MEDIATOR_DID_DOC.clone(), ALICE_DID_DOC.clone()]);
        let secret_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());
        const ALICE_DID: &str = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7";
        const MEDIATOR_DID: &str = "did:web:alice-mediator.com:alice_mediator_pub";
        let id = Uuid::new_v4().to_string();
        let msg: Message = Message::build(
            id,
            MEDIATE_FORWARD_2_0.to_string(),
            serde_json::json!(r#"next: "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"#),
        )
        .to(MEDIATOR_DID.to_owned())
        .from(ALICE_DID.to_owned())
        .finalize();

        let state = setup();
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
        let serialize_msg = serde_json::to_string(&msg).unwrap();

        // Mediator in action
        mediator_forward_process(state, serialize_msg).await.ok();
    }
}