use crate::error::ForwardError;
use database::Repository;
use didcomm::{AttachmentData, Message};
use mongodb::bson::doc;
use serde_json::{json, Value};
use shared::{
    repository::entity::{Connection, RoutedMessage},
    state::{AppState, AppStateRepository},
};
use std::sync::Arc;

async fn checks(
    message: &Message,
    connection_repository: &Arc<dyn Repository<Connection>>,
) -> Result<String, ForwardError> {
    let next = message.body.get("next").and_then(Value::as_str);
    match next {
        Some(next) => next,
        None => return Err(ForwardError::MalformedBody),
    };

    // Check if the client's did in mediator's keylist
    let _connection = match connection_repository
        .find_one_by(doc! {"keylist": doc!{ "$elemMatch": { "$eq": &next}}})
        .await
        .map_err(|_| ForwardError::InternalServerError)?
    {
        Some(connection) => connection,
        None => return Err(ForwardError::UncoordinatedSender),
    };

    Ok(next.unwrap().to_string())
}

pub(crate) async fn handler(state: Arc<AppState>, message: Message) -> Result<Option<Message>, ForwardError> {
    let AppStateRepository {
        message_repository,
        connection_repository,
        ..
    } = state
        .repository
        .as_ref()
        .ok_or_else(|| ForwardError::InternalServerError)?;

    let next = match checks(&message, connection_repository).await.ok() {
        Some(next) => Ok(next),
        None => Err(ForwardError::InternalServerError),
    };

    let attachments = message.attachments.unwrap_or_default();
    for attachment in attachments {
        let attached = match attachment.data {
            AttachmentData::Json { value: data } => data.json,
            AttachmentData::Base64 { value: data } => json!(data.base64),
            AttachmentData::Links { value: data } => json!(data.links),
        };
        message_repository
            .store(RoutedMessage {
                id: None,
                message: attached,
                recipient_did: next.as_ref().unwrap().to_owned(),
            })
            .await
            .map_err(|_| ForwardError::InternalServerError)?;
    }
    Ok(None)
}

#[cfg(test)]
mod test {
    use crate::web::handler::mediator_forward_process;

    use super::*;
    use did_utils::jwk::Jwk;
    use didcomm::{
        algorithms::AnonCryptAlg, protocols::routing::wrap_in_forward, secrets::SecretsResolver,
        Message, PackEncryptedOptions, UnpackOptions,
    };
    use keystore::tests::MockKeyStore;
    use keystore::Secrets;
    use serde_json::json;
    use shared::{
        repository::{
            entity::Connection,
            tests::{MockConnectionRepository, MockMessagesRepository},
        },
        state::AppStateRepository,
        utils::{resolvers::LocalSecretsResolver, tests_utils::tests},
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn _initial_connections() -> Vec<Connection> {
        let _recipient_did = _recipient_did();

        let connections = format!(
            r##"[
               {{
                    "client_did": "{_recipient_did}",
                    "mediator_did": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                    "routing_did": "did:key:generated",
                    "keylist": [
                        "{_recipient_did}"
                    ]
                }}
            ]"##
        );

        serde_json::from_str(&connections).unwrap()
    }

    #[tokio::test]
    async fn test_mediator_forward_process() {
        // simulate sender forwarding process
        let mut state = tests::setup().clone();
        let state = Arc::make_mut(&mut state);

        let mock_connections = MockConnectionRepository::from(_initial_connections());
        state.repository = Some(AppStateRepository {
            connection_repository: Arc::new(mock_connections),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
            keystore: Arc::new(MockKeyStore::new(vec![])),
        });

        let msg = Message::build(
            Uuid::new_v4().to_string(),
            "example/v1".to_owned(),
            json!("Hey there! Just wanted to remind you to step outside for a bit. A little fresh air can do wonders for your mood."),
        )
        .to(_recipient_did())
        .from(_sender_did())
        .finalize();

        let (packed_forward_msg, _metadata) = msg
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

        let msg = wrap_in_forward(
            &packed_forward_msg,
            None,
            &&_recipient_did(),
            &vec![_mediator_did(state)],
            &AnonCryptAlg::default(),
            &state.did_resolver,
        )
        .await
        .expect("Unable wrap_in_forward");

        let (msg, _metadata) = Message::unpack(
            &msg,
            &state.did_resolver,
            &state.secrets_resolver,
            &UnpackOptions::default(),
        )
        .await
        .expect("Unable unpack");

        let msg = mediator_forward_process(Arc::new(state.clone()), msg).await.unwrap();

        println!("Mediator1 is forwarding message \n{:?}\n", msg);
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

        let test_secret = Secrets {
            id: None,
            kid: secret_id.to_string(),
            secret_material: secret,
        };

        let keystore = Arc::new(MockKeyStore::new(vec![test_secret]));

        LocalSecretsResolver::new(keystore)
    }
}
