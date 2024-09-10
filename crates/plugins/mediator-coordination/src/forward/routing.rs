use didcomm::{Message, PackEncryptedOptions, UnpackOptions};

use crate::web::{error::MediationError, AppState, AppStateRepository};

pub async fn mediator_forward_process(
    payload: &str,
    state: &AppState,
) -> Result<(), MediationError> {
    // unpack encrypted payload message
    let AppStateRepository {
        message_repository, ..
    } = state
        .repository
        .as_ref()
        .expect("Missing Persistence Layer");

    let mediator_did = &state.diddoc.id;
    let (unpack_msg, _) = Message::unpack(
        payload,
        &state.did_resolver,
        &state.secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .map_err(|_| MediationError::MessageUnpackingFailure)?;
    if unpack_msg.to.is_some() {
        let dids = unpack_msg.clone().to.expect("to field is None");
        for did in dids {
            let (re_packed_msg, _) = unpack_msg
                .pack_encrypted(
                    &did,
                    Some(mediator_did),
                    None,
                    &state.did_resolver,
                    &state.secrets_resolver,
                    &PackEncryptedOptions::default(),
                )
                .await
                .expect("could not pack message: {0}");

            message_repository
                .store(serde_json::from_str(&re_packed_msg).expect("Could not deserialze packed message"))
                .await
                .map_err(|_| MediationError::PersisenceError)
                .unwrap();
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{borrow::Borrow, sync::Arc};

    use crate::{
        repository::stateful::coord::tests::{
            MockConnectionRepository, MockMessagesRepostiory, MockSecretsRepository,
        },
        util::{self, MockFileSystem},
        web::AppStateRepository,
    };

    use super::*;

    use didcomm::Message;
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            secret_repository: Arc::new(MockSecretsRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepostiory::from(vec![])),
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
        .from("did:web:alice-mediator.com:alice_mediator_pub".to_owned())
        .finalize();
        let serialize_msg = serde_json::to_string(msg.clone().borrow());
        let state = setup();

        mediator_forward_process(serialize_msg.unwrap().as_str(), &state)
            .await
            .ok();
    }
}


