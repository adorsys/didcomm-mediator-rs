use crate::{
    constants::{MESSAGE_DELIVERY_3_0, PROBLEM_REPORT_2_0, STATUS_RESPONSE_3_0},
    error::PickupError,
    model::{
        BodyDeliveryResponse, BodyLiveDeliveryChange, BodyStatusResponse, DeliveryResponse,
        LiveDeliveryChange, StatusResponse,
    },
};
use didcomm::{Attachment, Message, MessageBuilder};
use mongodb::bson::{doc, oid::ObjectId};
use serde_json::Value;
use shared::{
    midlw::ensure_transport_return_route_is_decorated_all,
    repository::entity::{Connection, RoutedMessage},
    state::{AppState, AppStateRepository},
};
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

// Process pickup status request
pub(crate) async fn handle_status_request(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, PickupError> {
    // Validate the return_route header
    ensure_transport_return_route_is_decorated_all(&message)
        .map_err(|_| PickupError::MalformedRequest("Missing return_route header".to_owned()))?;

    let mediator_did = &state.diddoc.id;
    let recipient_did = message
        .body
        .get("recipient_did")
        .and_then(|val| val.as_str());
    let sender_did = sender_did(&message)?;

    let repository = repository(state.clone())?;
    let connection = client_connection(&repository, sender_did).await?;

    let message_count = count_messages(repository, recipient_did, connection).await?;

    let id = Uuid::new_v4().urn().to_string();
    let response_builder: MessageBuilder = StatusResponse {
        id: id.as_str(),
        type_: STATUS_RESPONSE_3_0,
        body: BodyStatusResponse {
            recipient_did,
            message_count,
            live_delivery: Some(false),
            ..Default::default()
        },
    }
    .into();

    let response = response_builder
        .to(sender_did.to_owned())
        .from(mediator_did.to_owned())
        .finalize();

    Ok(Some(response))
}

// Process pickup delivery request
pub(crate) async fn handle_delivery_request(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, PickupError> {
    // Validate the return_route header
    ensure_transport_return_route_is_decorated_all(&message)
        .map_err(|_| PickupError::MalformedRequest("Missing return_route header".to_owned()))?;

    let mediator_did = &state.diddoc.id;
    let recipient_did = message
        .body
        .get("recipient_did")
        .and_then(|val| val.as_str());
    let sender_did = sender_did(&message)?;

    // Get the messages limit
    let limit = message
        .body
        .get("limit")
        .and_then(Value::as_u64)
        .ok_or_else(|| PickupError::MalformedRequest("Invalid \"limit\" specifier".to_owned()))?;

    let repository = repository(state.clone())?;
    let connection = client_connection(&repository, sender_did).await?;

    let messages = messages(repository, recipient_did, connection, limit as usize).await?;
    let id = Uuid::new_v4().urn().to_string();

    let response_builder: MessageBuilder = if messages.is_empty() {
        StatusResponse {
            id: id.as_str(),
            type_: STATUS_RESPONSE_3_0,
            body: BodyStatusResponse {
                recipient_did,
                message_count: 0,
                live_delivery: Some(false),
                ..Default::default()
            },
        }
        .into()
    } else {
        let mut attachments: Vec<Attachment> = Vec::with_capacity(messages.len());

        for message in messages {
            let attached = Attachment::json(message.message)
                .id(message.id.map(|id| id.to_string()).ok_or_else(|| {
                    PickupError::InternalError(
                        "Failed to load requested messages. Please try again later.".to_owned(),
                    )
                })?)
                .finalize();

            attachments.push(attached);
        }

        DeliveryResponse {
            id: id.as_str(),
            thid: id.as_str(),
            type_: MESSAGE_DELIVERY_3_0,
            body: BodyDeliveryResponse { recipient_did },
            attachments,
        }
        .into()
    };

    let response = response_builder
        .to(sender_did.to_owned())
        .from(mediator_did.to_owned())
        .finalize();

    Ok(Some(response))
}

// Process pickup messages acknowledgement
pub(crate) async fn handle_message_acknowledgement(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, PickupError> {
    // Validate the return_route header
    ensure_transport_return_route_is_decorated_all(&message)
        .map_err(|_| PickupError::MalformedRequest("Missing return_route header".to_owned()))?;

    let mediator_did = &state.diddoc.id;
    let repository = repository(state.clone())?;
    let sender_did = sender_did(&message)?;
    let connection = client_connection(&repository, sender_did).await?;

    // Get the message id list
    let message_id_list = message
        .body
        .get("message_id_list")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .ok_or_else(|| {
            PickupError::MalformedRequest("Invalid \"message_id_list\" specifier".to_owned())
        })?;

    for id in message_id_list {
        let msg_id = ObjectId::from_str(id);
        if msg_id.is_err() {
            return Err(PickupError::MalformedRequest(format!(
                "Invalid message id: {id}"
            )));
        }
        repository
            .message_repository
            .delete_one(msg_id.unwrap())
            .await
            .map_err(|_| {
                PickupError::InternalError(
                    "Failed to process the request. Please try again later.".to_owned(),
                )
            })?;
    }

    let message_count = count_messages(repository, None, connection).await?;

    let id = Uuid::new_v4().urn().to_string();
    let response_builder: MessageBuilder = StatusResponse {
        id: id.as_str(),
        type_: STATUS_RESPONSE_3_0,
        body: BodyStatusResponse {
            message_count,
            live_delivery: Some(false),
            ..Default::default()
        },
    }
    .into();

    let response = response_builder
        .to(sender_did.to_owned())
        .from(mediator_did.to_owned())
        .finalize();

    Ok(Some(response))
}

// Process live delivery change request
pub(crate) async fn handle_live_delivery_change(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, PickupError> {
    // Validate the return_route header
    ensure_transport_return_route_is_decorated_all(&message)
        .map_err(|_| PickupError::MalformedRequest("Missing return_route header".to_owned()))?;

    match message.body.get("live_delivery").and_then(Value::as_bool) {
        Some(true) => {
            let mediator_did = &state.diddoc.id;
            let sender_did = sender_did(&message)?;
            let id = Uuid::new_v4().urn().to_string();
            let pthid = message.thid.as_deref().unwrap_or(id.as_str());

            let response_builder: MessageBuilder = LiveDeliveryChange {
                id: id.as_str(),
                pthid,
                type_: PROBLEM_REPORT_2_0,
                body: BodyLiveDeliveryChange {
                    code: "e.m.live-mode-not-supported",
                    comment: "Connection does not support Live Delivery",
                },
            }
            .into();

            let response = response_builder
                .to(sender_did.to_owned())
                .from(mediator_did.to_owned())
                .finalize();

            Ok(Some(response))
        }
        Some(false) => Ok(None),
        None => Err(PickupError::MalformedRequest(
            "Missing \"live_delivery\" specifier".to_owned(),
        )),
    }
}

async fn count_messages(
    repository: AppStateRepository,
    recipient_did: Option<&str>,
    connection: Connection,
) -> Result<usize, PickupError> {
    let recipients = recipients(recipient_did, &connection);

    let count = repository
        .message_repository
        .count_by(doc! { "recipient_did": { "$in": recipients } })
        .await
        .map_err(|_| {
            PickupError::InternalError(
                "Failed to process the request. Please try again later.".to_owned(),
            )
        })?;

    Ok(count)
}

async fn messages(
    repository: AppStateRepository,
    recipient_did: Option<&str>,
    connection: Connection,
    limit: usize,
) -> Result<Vec<RoutedMessage>, PickupError> {
    let recipients = recipients(recipient_did, &connection);

    let routed_messages = repository
        .message_repository
        .find_all_by(
            doc! { "recipient_did": { "$in": recipients } },
            Some(limit as i64),
        )
        .await
        .map_err(|_| {
            PickupError::InternalError(
                "Failed to process the request. Please try again later.".to_owned(),
            )
        })?;

    Ok(routed_messages)
}

#[inline]
fn recipients<'a>(recipient_did: Option<&'a str>, connection: &'a Connection) -> Vec<&'a str> {
    recipient_did
        .map(|did| {
            if connection.keylist.contains(&did.to_owned()) {
                vec![did]
            } else {
                Vec::new()
            }
        })
        .unwrap_or_else(|| connection.keylist.iter().map(|s| s.as_str()).collect())
}

#[inline]
fn sender_did(message: &Message) -> Result<&str, PickupError> {
    message.from.as_deref().ok_or(PickupError::MissingSenderDID)
}

#[inline]
fn repository(state: Arc<AppState>) -> Result<AppStateRepository, PickupError> {
    state.repository.clone().ok_or(PickupError::InternalError(
        "Internal server error. Please try again later.".to_owned(),
    ))
}

#[inline]
async fn client_connection(
    repository: &AppStateRepository,
    client_did: &str,
) -> Result<Connection, PickupError> {
    repository
        .connection_repository
        .find_one_by(doc! { "client_did": client_did })
        .await
        .map_err(|_| {
            PickupError::InternalError(
                "Failed to process the request. Please try again later.".to_owned(),
            )
        })?
        .ok_or(PickupError::MissingClientConnection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        DELIVERY_REQUEST_3_0, LIVE_MODE_CHANGE_3_0, MESSAGE_DELIVERY_3_0, MESSAGE_RECEIVED_3_0,
        PROBLEM_REPORT_2_0, STATUS_REQUEST_3_0, STATUS_RESPONSE_3_0,
    };
    use serde_json::json;
    use shared::{
        repository::tests::{MockConnectionRepository, MockMessagesRepository},
        utils::tests_utils::tests as global,
    };

    #[allow(clippy::needless_update)]
    fn setup(connections: Vec<Connection>, stored_messages: Vec<RoutedMessage>) -> Arc<AppState> {
        let state = global::setup();
        let mut state = Arc::into_inner(state).unwrap();

        state.repository = Some(AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(connections)),
            message_repository: Arc::new(MockMessagesRepository::from(stored_messages)),
            ..state.repository.take().unwrap()
        });

        Arc::new(state)
    }

    fn test_connections() -> Vec<Connection> {
        vec![Connection {
            id: Some(ObjectId::from_str("6580701fd2d92bb3cd291b2a").unwrap()),
            client_did: "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_owned(),
            mediator_did: "did:web:alice-mediator.com:alice_mediator_pub".to_owned(),
            routing_did: "did:peer:mediator_generated".to_owned(),
            keylist: vec!["did:key:alice_identity_pub@alice_mediator".to_owned()],
        }]
    }

    fn test_messages() -> Vec<RoutedMessage> {
        vec![
            RoutedMessage {
                id: Some(ObjectId::from_str("6580701fd2d92bb3cd291b2a").unwrap()),
                recipient_did: "did:key:alice_identity_pub@alice_mediator".to_owned(),
                message: json!("test1"),
            },
            RoutedMessage {
                id: Some(ObjectId::from_str("7589601fd2d92bb3cd291b2a").unwrap()),
                recipient_did: "did:key:alice_identity_pub@alice_mediator".to_owned(),
                message: json!("test2"),
            },
            RoutedMessage {
                id: Some(ObjectId::from_str("9651201fd2d92bb3cd291b2a").unwrap()),
                recipient_did: "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"
                    .to_owned(),
                message: json!("test3"),
            },
        ]
    }

    #[tokio::test]
    async fn test_handle_status_request_success_with_messages() {
        let state = setup(test_connections(), test_messages());

        // Expect to receive 2 message count for this recipient_did
        let request = Message::build(
            "id_alice_message_status_request".to_owned(),
            STATUS_REQUEST_3_0.to_owned(),
            json!({"recipient_did": "did:key:alice_identity_pub@alice_mediator"}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let response = handle_status_request(Arc::clone(&state), request)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(response.type_, STATUS_RESPONSE_3_0);
        assert_eq!(response.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(response.to.unwrap(), vec![global::_edge_did()]);
        assert_eq!(
            response.body,
            json!({"recipient_did": "did:key:alice_identity_pub@alice_mediator", "message_count": 2, "live_delivery": false})
        );
    }

    #[tokio::test]
    async fn test_handle_status_request_success_with_no_messages() {
        let state = setup(test_connections(), test_messages());

        // Expect to receive 0 message count for this recipient_did since it is not in the keylist
        let request = Message::build(
            "id_alice_message_status_request".to_owned(),
            STATUS_REQUEST_3_0.to_owned(),
            json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let response = handle_status_request(Arc::clone(&state), request)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(
            response.body,
            json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7", "message_count": 0, "live_delivery": false})
        );
    }

    #[tokio::test]
    async fn test_handle_status_request_with_no_recipient_did_specified() {
        let state = setup(test_connections(), test_messages());

        // Expect to receive 2 message count since the recipient_did is not specified
        // so it will return the number of messages for all dids in the keylist for that sender connection
        let request = Message::build(
            "id_alice_message_status_request".to_owned(),
            "https://didcomm.org/messagepickup/3.0/status-request".to_owned(),
            json!({}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let response = handle_status_request(Arc::clone(&state), request)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(
            response.body,
            json!({"message_count": 2, "live_delivery": false})
        );
    }

    #[tokio::test]
    async fn test_handle_status_request_failed_with_no_recipient_connection() {
        let state = setup(test_connections(), test_messages());

        // Should return an error if the recipient has no connection with the mediator
        let invalid_request = Message::build(
            "id_alice_message_status_request".to_owned(),
            STATUS_REQUEST_3_0.to_owned(),
            json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from("did:key:invalid".to_owned())
        .finalize();

        let error = handle_status_request(state, invalid_request)
            .await
            .unwrap_err();

        assert_eq!(error, PickupError::MissingClientConnection);
    }

    #[tokio::test]
    async fn test_handle_delivery_request_with_recipient_did_in_keylist() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_message_delivery_request".to_owned(),
            DELIVERY_REQUEST_3_0.to_owned(),
            json!({"recipient_did": "did:key:alice_identity_pub@alice_mediator", "limit": 5}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let response = handle_delivery_request(Arc::clone(&state), request)
            .await
            .unwrap()
            .expect("Response should not be None");

        let expected_attachments = vec![
            Attachment::json(json!("test1"))
                .id(ObjectId::from_str("6580701fd2d92bb3cd291b2a")
                    .unwrap()
                    .to_string())
                .finalize(),
            Attachment::json(json!("test2"))
                .id(ObjectId::from_str("7589601fd2d92bb3cd291b2a")
                    .unwrap()
                    .to_string())
                .finalize(),
        ];

        assert_eq!(response.thid.unwrap(), response.id);
        assert_eq!(response.type_, MESSAGE_DELIVERY_3_0);
        assert_eq!(response.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(response.to.unwrap(), vec![global::_edge_did()]);
        assert_eq!(
            response.body,
            json!({"recipient_did": "did:key:alice_identity_pub@alice_mediator"})
        );
        assert_eq!(response.attachments.unwrap(), expected_attachments);
    }

    #[tokio::test]
    async fn test_handle_delivery_request_with_recipient_did_not_in_keylist() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_message_delivery_request".to_owned(),
            DELIVERY_REQUEST_3_0.to_owned(),
            json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7", "limit": 5}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // When the specified recipient did is not in the keylist,
        // it should return a status response with a message count of 0
        let response = handle_delivery_request(state, request)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(response.type_, STATUS_RESPONSE_3_0);
        assert_eq!(
            response.body,
            json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7", "message_count": 0, "live_delivery": false})
        );
    }

    #[tokio::test]
    async fn test_handle_delivery_request_with_limit_set_to_0() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_message_delivery_request".to_owned(),
            DELIVERY_REQUEST_3_0.to_owned(),
            json!({"limit": 0}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // When the limit is set to 0, it should return all the messages in the queue
        // and since the recipient did is not specified, it should return the messages
        // for all the dids in the keylist for that sender connection
        let response = handle_delivery_request(state, request)
            .await
            .unwrap()
            .expect("Response should not be None");

        let expected_attachments = vec![
            Attachment::json(json!("test1"))
                .id(ObjectId::from_str("6580701fd2d92bb3cd291b2a")
                    .unwrap()
                    .to_string())
                .finalize(),
            Attachment::json(json!("test2"))
                .id(ObjectId::from_str("7589601fd2d92bb3cd291b2a")
                    .unwrap()
                    .to_string())
                .finalize(),
        ];

        assert_eq!(response.type_, MESSAGE_DELIVERY_3_0);
        assert_eq!(response.body, json!({}));
        assert_eq!(response.attachments.unwrap(), expected_attachments);
    }

    #[tokio::test]
    async fn test_handle_delivery_request_with_limit_set_to_1() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_message_delivery_request".to_owned(),
            DELIVERY_REQUEST_3_0.to_owned(),
            json!({"limit": 1}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Since the recipient did is not specified, it should return the messages
        // for all the dids in the keylist for that sender connection (2 in this case)
        // The limit is set to 1 so it should return the first message in the queue
        let response = handle_delivery_request(state, request)
            .await
            .unwrap()
            .expect("Response should not be None");

        let expected_attachments = vec![Attachment::json(json!("test1"))
            .id(ObjectId::from_str("6580701fd2d92bb3cd291b2a")
                .unwrap()
                .to_string())
            .finalize()];

        assert_eq!(response.type_, MESSAGE_DELIVERY_3_0);
        assert_eq!(response.body, json!({}));
        assert_eq!(response.attachments.unwrap(), expected_attachments);
    }

    #[tokio::test]
    async fn test_handle_message_acknowledgement_with_invalid_message_id() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_message_aknowledgement".to_owned(),
            MESSAGE_RECEIVED_3_0.to_owned(),
            json!({"message_id_list": ["6689601fd2d92bb3cd451b2c","6389601fd2d92bb3cd451b2d"]}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Should return 2 since these ids are not associated with any message
        let response = handle_message_acknowledgement(state, request)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(response.type_, STATUS_RESPONSE_3_0);
        assert_eq!(
            response.body,
            json!({"message_count": 2, "live_delivery": false})
        );
    }

    #[tokio::test]
    async fn test_handle_message_acknowledgement_with_valid_message_id() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_message_aknowledgement".to_owned(),
            MESSAGE_RECEIVED_3_0.to_owned(),
            json!({"message_id_list": ["6580701fd2d92bb3cd291b2a", "6689601fd2d92bb3cd451b2c"]}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Should return 1 since one id in the list is associated
        // to the first message in the queue and then will be deleted
        let response = handle_message_acknowledgement(state, request)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(response.type_, STATUS_RESPONSE_3_0);
        assert_eq!(
            response.body,
            json!({"message_count": 1, "live_delivery": false})
        );
    }

    #[tokio::test]
    async fn test_handle_live_delivery_change() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_live_delivery_change".to_owned(),
            LIVE_MODE_CHANGE_3_0.to_owned(),
            json!({"live_delivery": true}),
        )
        .thid("123".to_owned())
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let response = handle_live_delivery_change(state, request)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(response.pthid.unwrap(), "123");
        assert_eq!(response.type_, PROBLEM_REPORT_2_0);
        assert_eq!(
            response.body,
            json!({
                "code": "e.m.live-mode-not-supported",
                "comment": "Connection does not support Live Delivery"
            })
        );
    }

    #[tokio::test]
    async fn test_handle_live_delivery_change_false() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_live_delivery_change".to_owned(),
            LIVE_MODE_CHANGE_3_0.to_owned(),
            json!({"live_delivery": false}),
        )
        .thid("123".to_owned())
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let response = handle_live_delivery_change(state, request).await.unwrap();

        assert_eq!(response, None);
    }

    #[tokio::test]
    async fn test_handle_live_delivery_change_with_invalid_body() {
        let state = setup(test_connections(), test_messages());

        let request = Message::build(
            "id_alice_live_delivery_change".to_owned(),
            LIVE_MODE_CHANGE_3_0.to_owned(),
            json!({}),
        )
        .thid("123".to_owned())
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let error = handle_live_delivery_change(state, request)
            .await
            .unwrap_err();

        assert_eq!(
            error,
            PickupError::MalformedRequest("Missing \"live_delivery\" specifier".to_owned())
        );
    }
}
