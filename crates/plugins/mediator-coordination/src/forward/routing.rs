use axum::response::{IntoResponse, Response};
use didcomm::{protocols::routing::try_parse_forward, AttachmentData, Message};

use hyper::StatusCode;
use mongodb::bson::doc;
use serde_json::{json, Value};

use crate::{
    model::stateful::entity::RoutedMessage,
    web::{error::MediationError, AppState, AppStateRepository},
};

/// mediator receives messages of type forward then it unpacks the messages and stores it for pickup
/// the unpacked message is then repacked for further transmission.
pub async fn mediator_forward_process(
    state: &AppState,
    payload: Message,
) -> Result<Message, Response> {
    let AppStateRepository {
        message_repository,
        connection_repository,
        ..
    } = state
        .repository
        .as_ref()
        .ok_or_else(|| MediationError::RepostitoryError)
        .unwrap();

    // Check if the client's did in mediator's keylist

    let next = payload.body.get("next").and_then(Value::as_str).unwrap();
            
    let _connection = match connection_repository
        .find_one_by(doc! {"keylist": doc!{ "$elemMatch": { "$eq": &next}}})
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

    let attachments = payload.attachments.unwrap_or_default();
    for att in attachments {
        let attached = match att.data {
            AttachmentData::Json { value: val} => val.json,
            _ => json!(0)
        };
        message_repository
            .store(RoutedMessage {
                id: None,
                message: json!(attached),
                recipient_did: next.to_string(),
            })
            .await
            .map_err(|_| MediationError::PersisenceError)
            .unwrap();
    }

// let result = try_parse_forward(&payload).unwrap();
//     let _connection = match connection_repository
//         .find_one_by(doc! {"keylist": doc!{ "$elemMatch": { "$eq": &result.next}}})
//         .await
//         .unwrap()
    // {
    //     Some(connection) => connection,
    //     None => {
    //         let response = (
    //             StatusCode::UNAUTHORIZED,
    //             MediationError::UncoordinatedSender.json(),
    //         );
    //         return Err(response.into_response());
    //     }
    // };

    // store message attachement with associated recipient did

    // let forward_msg = serde_json::to_string(&result.forwarded_msg).unwrap();

    // let messages = RoutedMessage {
    //     id: None,
    //     message: forward_msg,
    //     recipient_did: result.next,
    // };
    // message_repository
    //     .store(messages)
    //     .await
    //     .map_err(|_| MediationError::PersisenceError)
    //     .unwrap();
    // Ok(result.msg.to_owned())
    Ok(Message::build("".to_string(), "".to_string(),json!("")).finalize())
}

#[cfg(test)]
mod test {

    use std::sync::Arc;

    use crate::{
        didcomm::bridge::LocalSecretsResolver,
        model::stateful::entity::Connection,
        repository::stateful::tests::{
            MockConnectionRepository, MockMessagesRepository, MockSecretsRepository,
        },
        util::{self, MockFileSystem},
        web::AppStateRepository,
    };

    use super::*;

    use did_utils::jwk::Jwk;
    use didcomm::{
        algorithms::AnonCryptAlg, protocols::routing::wrap_in_forward, secrets::SecretsResolver,
        Message, PackEncryptedOptions, UnpackOptions,
    };
    use serde_json::json;

    use uuid::Uuid;
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let storage_dirpath = std::env::var("STORAGE_DIRPATH").unwrap_or_else(|_| "/".to_owned());
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

    #[tokio::test]
    async fn test_mediator_forward_process() {
        _initial_connections();
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
        println!("Encryption metadata is\n{:?}\n", _metadata);

        // --- Sending message by Alice ---
        println!("Alice is sending message \n{}\n", packed_forward_msg);

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

        println!(" wraped in forward\n{}\n", msg);
        let (msg, _metadata) = Message::unpack(
            &msg,
            &state.did_resolver,
            &state.secrets_resolver,
            &UnpackOptions::default(),
        )
        .await
        .expect("Unable unpack");

        println!("Mediator1 received message is \n{:?}\n", msg);

        println!(
            "Mediator1 received message unpack metadata is \n{:?}\n",
            _metadata
        );

        let msg = mediator_forward_process(state, msg).await.unwrap();

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

        LocalSecretsResolver::new(&secret_id, &secret)
    }
}