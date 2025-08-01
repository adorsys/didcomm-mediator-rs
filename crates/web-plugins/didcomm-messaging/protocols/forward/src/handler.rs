use crate::error::ForwardError;
use database::Repository;
use didcomm::{AttachmentData, Message};
use futures::future::try_join_all;
use mongodb::bson::doc;
use serde_json::{json, Value};
use shared::{
    breaker::{CircuitBreaker, Error as BreakerError},
    repository::entity::{Connection, RoutedMessage},
    state::AppState,
};
use std::sync::Arc;

/// Mediator receives forwarded messages, extract the next field in the message body, and the attachments in the message
/// then stores the attachment with the next field as key for pickup
pub(crate) async fn mediator_forward_process(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, ForwardError> {
    // Check if the circuit breaker is open
    state
        .db_circuit_breaker
        .should_allow_call()
        .then_some(())
        .ok_or(ForwardError::ServiceUnavailable)?;

    let repository = state
        .repository
        .as_ref()
        .ok_or(ForwardError::InternalServerError)?;

    let next = checks(
        &message,
        &repository.connection_repository,
        &state.db_circuit_breaker,
    )
    .await
    .map_err(|_| ForwardError::InternalServerError)?;

    let attachments = message.attachments.unwrap_or_default();
    let store_futures: Vec<_> = attachments
        .into_iter()
        .map(|attachment| async {
            let attached = match attachment.data {
                AttachmentData::Json { value } => value.json,
                AttachmentData::Base64 { value } => json!(value.base64),
                AttachmentData::Links { value } => json!(value.links),
            };

            let routed_message = RoutedMessage {
                id: None,
                message: attached,
                recipient_did: next.clone(),
            };

            state
                .db_circuit_breaker
                .call(|| {
                    repository
                        .message_repository
                        .store(routed_message.to_owned())
                })
                .await
                .map_err(|err| match err {
                    BreakerError::CircuitOpen => ForwardError::ServiceUnavailable,
                    _ => ForwardError::InternalServerError,
                })
        })
        .collect();

    try_join_all(store_futures).await?;
    Ok(None)
}

async fn checks(
    message: &Message,
    connection_repository: &Arc<dyn Repository<Connection>>,
    circuit_breaker: &CircuitBreaker,
) -> Result<String, ForwardError> {
    let next = match message.body.get("next") {
        Some(Value::String(next)) => next.clone(),
        _ => return Err(ForwardError::MalformedBody),
    };

    // Check if the client's did is in mediator's keylist
    circuit_breaker
        .call(|| {
            connection_repository
                .find_one_by(doc! {"keylist": doc!{ "$elemMatch": { "$eq": &next}}})
        })
        .await
        .map_err(|err| match err {
            BreakerError::CircuitOpen => ForwardError::ServiceUnavailable,
            BreakerError::Inner(err) => {
                tracing::error!("Failed to find connection: {err:?}");
                ForwardError::InternalServerError
            }
        })?
        .is_some()
        .then_some(())
        .ok_or(ForwardError::UncoordinatedSender)?;

    Ok(next)
}

#[cfg(test)]
mod test {
    use super::*;
    use did_utils::jwk::Jwk;
    use didcomm::{
        algorithms::AnonCryptAlg, protocols::routing::wrap_in_forward, secrets::SecretsResolver,
        Message, PackEncryptedOptions, UnpackOptions,
    };
    use keystore::Keystore;
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
            keystore: Keystore::new(),
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
            &_recipient_did(),
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

        let msg = mediator_forward_process(Arc::new(state.clone()), msg)
            .await
            .unwrap();

        println!("Mediator1 is forwarding message \n{msg:?}\n");
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

        let keystore = Keystore::with_mock_configs(vec![(secret_id.to_string(), secret)]);

        LocalSecretsResolver::new(keystore)
    }
}
