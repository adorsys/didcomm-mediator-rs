use std::{fs, sync::Arc};

use didcomm::{error::Error, Message, PackEncryptedOptions, UnpackOptions};

use crate::web::{error::MediationError, AppState};

struct MessageStore {
    dirpath: String,
}
impl MessageStore {
    fn persist_msg(&self, content: String, inode: String) {
        fs::create_dir_all(&self.dirpath).unwrap();
        let file = format!("{}/{}.json", &self.dirpath, inode,);
        fs::write(file, content).unwrap();
    }
}

pub async fn mediator_forward_process(
    mediator_did: Option<&str>,
    payload: &str,
    state: &AppState,
    store_dir_path: String,
) -> Result<(), MediationError> {
    // unpack encrypted payload message
    let store = MessageStore {
        dirpath: store_dir_path,
    };

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
                    mediator_did,
                    None,
                    &state.did_resolver,
                    &state.secrets_resolver,
                    &PackEncryptedOptions::default(),
                )
                .await
                .expect("could not pack message: {0}");

            store.persist_msg(serde_json::to_string_pretty(&re_packed_msg).unwrap(), did)
        }
    }

    Ok(())
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

    use didcomm::Message;
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

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

        // case where  mediator did is not provided
        let _pickup_msg = mediator_forward_process(
            None,
            serialize_msg.unwrap().as_str(),
            &state,
            "./msg".to_string(),
        )
        .await
        .unwrap();
    }
}
