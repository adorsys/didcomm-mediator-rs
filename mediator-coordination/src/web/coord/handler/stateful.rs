use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use didcomm::Message;
use mongodb::bson::doc;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    constant::KEYLIST_UPDATE_RESPONSE_2_0,
    model::stateful::coord::{
        entity::Connection, KeylistUpdateAction, KeylistUpdateBody, KeylistUpdateConfirmation,
        KeylistUpdateResponseBody, KeylistUpdateResult,
    },
    web::{error::MediationError, AppState, AppStateRepository},
};

pub async fn process_plain_keylist_update_message(
    state: Arc<AppState>,
    message: Message,
) -> Result<Message, Response> {
    // Extract message sender

    let sender = message
        .from
        .expect("unpacking middleware failed to prevent anonymous senders");

    // Parse message body into keylist update

    let keylist_update_body: KeylistUpdateBody = match serde_json::from_value(message.body) {
        Ok(serialized) => serialized,
        Err(_) => {
            let response = (
                StatusCode::BAD_REQUEST,
                MediationError::UnexpectedMessageFormat.json(),
            );

            return Err(response.into_response());
        }
    };

    // Retrieve repository to connection entities

    let AppStateRepository {
        connection_repository,
        ..
    } = state
        .repository
        .as_ref()
        .expect("missing persistence layer");

    // Find connection for this keylist update

    let connection = match connection_repository
        .find_one_by(doc! { "client_did": &sender })
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

    // Prepare handles to relevant collections

    let mut updated_keylist = connection.keylist.clone();
    let updates = keylist_update_body.updates;

    // Closure to check if a specific key is duplicated across commands

    let key_is_duplicate = |recipient_did| {
        updates
            .iter()
            .filter(|e| &e.recipient_did == recipient_did)
            .count()
            > 1
    };

    // Perform updates to persist

    let confirmations: Vec<_> = updates
        .iter()
        .map(|update| KeylistUpdateConfirmation {
            recipient_did: update.recipient_did.clone(),
            action: update.action.clone(),
            result: {
                if let KeylistUpdateAction::Unknown(_) = &update.action {
                    KeylistUpdateResult::ClientError
                } else if key_is_duplicate(&update.recipient_did) {
                    KeylistUpdateResult::ClientError
                } else {
                    match connection
                        .keylist
                        .iter()
                        .position(|x| x == &update.recipient_did)
                    {
                        Some(index) => match &update.action {
                            KeylistUpdateAction::Add => KeylistUpdateResult::NoChange,
                            KeylistUpdateAction::Remove => {
                                updated_keylist.swap_remove(index);
                                KeylistUpdateResult::Success
                            }
                            KeylistUpdateAction::Unknown(_) => unreachable!(),
                        },
                        None => match &update.action {
                            KeylistUpdateAction::Add => {
                                updated_keylist.push(update.recipient_did.clone());
                                KeylistUpdateResult::Success
                            }
                            KeylistUpdateAction::Remove => KeylistUpdateResult::NoChange,
                            KeylistUpdateAction::Unknown(_) => unreachable!(),
                        },
                    }
                }
            },
        })
        .collect();

    // Persist updated keylist, update confirmations if server error

    let confirmations = match connection_repository
        .update(Connection {
            keylist: updated_keylist,
            ..connection
        })
        .await
    {
        Ok(_) => confirmations,
        Err(_) => confirmations
            .into_iter()
            .map(|mut confirmation| {
                if confirmation.result != KeylistUpdateResult::ClientError {
                    confirmation.result = KeylistUpdateResult::ServerError
                }

                confirmation
            })
            .collect(),
    };

    // Build response

    let mediator_did = &state.diddoc.id;

    Ok(Message::build(
        format!("urn:uuid:{}", Uuid::new_v4()),
        KEYLIST_UPDATE_RESPONSE_2_0.to_string(),
        json!(KeylistUpdateResponseBody {
            updated: confirmations
        }),
    )
    .to(sender.clone())
    .from(mediator_did.clone())
    .finalize())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    use crate::{
        repository::stateful::coord::tests::MockConnectionRepository, web::handler::tests as global,
    };

    #[allow(clippy::needless_update)]
    fn setup(initial_connections: Vec<Connection>) -> Arc<AppState> {
        let (_, state) = global::setup();

        let mut state = match Arc::try_unwrap(state) {
            Ok(state) => state,
            Err(_) => panic!(),
        };

        state.repository = Some(AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(initial_connections)),
            ..state.repository.unwrap()
        });

        Arc::new(state)
    }

    #[tokio::test]
    async fn test_keylist_update() {
        let state = setup(_initial_connections());

        // Prepare request

        let message = Message::build(
            "id_alice_keylist_update_request".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
            json!({
                "updates": [
                    {
                        "action": "remove",
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    },
                    {
                        "action": "add",
                        "recipient_did": "did:key:alice_identity_pub2@alice_mediator"
                    },
                ]
            }),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Process request

        let response = process_plain_keylist_update_message(Arc::clone(&state), message)
            .await
            .unwrap();

        // Assert metadata

        assert_eq!(response.type_, KEYLIST_UPDATE_RESPONSE_2_0);
        assert_eq!(response.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(response.to.unwrap(), vec![global::_edge_did()]);

        // Assert updates

        assert_eq!(
            response.body,
            json!({
                "updated": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                        "action": "remove",
                        "result": "success"
                    },
                    {
                        "recipient_did":"did:key:alice_identity_pub2@alice_mediator",
                        "action": "add",
                        "result": "success"
                    },
                ]
            })
        );

        // Assert repository state

        let AppStateRepository {
            connection_repository,
            ..
        } = state.repository.as_ref().unwrap();

        let connections = connection_repository.find_all().await.unwrap();
        assert_eq!(
            connections,
            serde_json::from_str::<Vec<Connection>>(
                r##"[
                    {
                        "_id": {
                            "$oid": "6580701fd2d92bb3cd291b2a"
                        },
                        "client_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                        "mediator_did": "did:web:alice-mediator.com:alice_mediator_pub",
                        "keylist": [
                            "did:key:alice_identity_pub2@alice_mediator"
                        ]
                    },
                    {
                        "_id": {
                            "$oid": "6580701fd2d92bb3cd291b2b"
                        },
                        "client_did": "did:key:other",
                        "mediator_did": "did:web:alice-mediator.com:alice_mediator_pub",
                        "keylist": []
                    }
                ]"##
            )
            .unwrap()
        );
    }

    #[tokio::test]
    async fn test_keylist_update_no_change() {
        let state = setup(_initial_connections());

        // Prepare request

        let message = Message::build(
            "id_alice_keylist_update_request".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
            json!({
                "updates": [
                    {
                        "action": "add",
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    },
                    {
                        "action": "remove",
                        "recipient_did": "did:key:alice_identity_pub2@alice_mediator"
                    },
                ]
            }),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Process request

        let response = process_plain_keylist_update_message(Arc::clone(&state), message)
            .await
            .unwrap();

        // Assert updates

        assert_eq!(
            response.body,
            json!({
                "updated": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                        "action": "add",
                        "result": "no_change"
                    },
                    {
                        "recipient_did":"did:key:alice_identity_pub2@alice_mediator",
                        "action": "remove",
                        "result": "no_change"
                    },
                ]
            })
        );

        // Assert repository state

        let AppStateRepository {
            connection_repository,
            ..
        } = state.repository.as_ref().unwrap();

        let connections = connection_repository.find_all().await.unwrap();
        assert_eq!(connections, _initial_connections());
    }

    #[tokio::test]
    async fn test_keylist_update_duplicate_results_in_client_error() {
        let state = setup(_initial_connections());

        // Prepare request

        let message = Message::build(
            "id_alice_keylist_update_request".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
            json!({
                "updates": [
                    {
                        "action": "add",
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    },
                    {
                        "action": "remove",
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    },
                ]
            }),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Process request

        let response = process_plain_keylist_update_message(Arc::clone(&state), message)
            .await
            .unwrap();

        // Assert updates

        assert_eq!(
            response.body,
            json!({
                "updated": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                        "action": "add",
                        "result": "client_error"
                    },
                    {
                        "recipient_did":"did:key:alice_identity_pub1@alice_mediator",
                        "action": "remove",
                        "result": "client_error"
                    },
                ]
            })
        );

        // Assert repository state

        let AppStateRepository {
            connection_repository,
            ..
        } = state.repository.as_ref().unwrap();

        let connections = connection_repository.find_all().await.unwrap();
        assert_eq!(connections, _initial_connections());
    }

    #[tokio::test]
    async fn test_keylist_update_unknown_action_results_in_client_error() {
        let state = setup(_initial_connections());

        // Prepare request

        let message = Message::build(
            "id_alice_keylist_update_request".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
            json!({
                "updates": [
                    {
                        "action": "unknown",
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    }
                ]
            }),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Process request

        let response = process_plain_keylist_update_message(Arc::clone(&state), message)
            .await
            .unwrap();

        // Assert updates

        assert_eq!(
            response.body,
            json!({
                "updated": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                        "action": "unknown",
                        "result": "client_error"
                    }
                ]
            })
        );
    }

    #[tokio::test]
    async fn test_keylist_update_with_malformed_request() {
        let state = setup(_initial_connections());

        // Prepare request
        let message = Message::build(
            "id_alice_keylist_update_request".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
            json!("not-keylist-update-request"),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Process request
        let err = process_plain_keylist_update_message(Arc::clone(&state), message)
            .await
            .unwrap_err();

        // Assert issued error
        _assert_delegate_handler_err(
            err,
            StatusCode::BAD_REQUEST,
            MediationError::UnexpectedMessageFormat,
        )
        .await;
    }

    #[tokio::test]
    async fn test_keylist_update_unknown_sender_results_in_unauthorized_error() {
        let state = setup(
            serde_json::from_str(
                r##"[
                {
                    "_id": {
                        "$oid": "6580701fd2d92bb3cd291b2a"
                    },
                    "client_did": "did:key:alt",
                    "mediator_did": "did:web:alice-mediator.com:alice_mediator_pub",
                    "keylist": []
                }
            ]"##,
            )
            .unwrap(),
        );

        // Prepare request
        let message = Message::build(
            "id_alice_keylist_update_request".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
            json!({
                "updates": [
                    {
                        "action": "add",
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    }
                ]
            }),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Process request
        let err = process_plain_keylist_update_message(Arc::clone(&state), message)
            .await
            .unwrap_err();

        // Assert issued error
        _assert_delegate_handler_err(
            err,
            StatusCode::UNAUTHORIZED,
            MediationError::UncoordinatedSender,
        )
        .await;
    }

    //----------------------------------------------------------------------------------------------
    // Helpers -------------------------------------------------------------------------------------
    //----------------------------------------------------------------------------------------------

    fn _initial_connections() -> Vec<Connection> {
        serde_json::from_str(
            r##"[
                {
                    "_id": {
                        "$oid": "6580701fd2d92bb3cd291b2a"
                    },
                    "client_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                    "mediator_did": "did:web:alice-mediator.com:alice_mediator_pub",
                    "keylist": [
                        "did:key:alice_identity_pub1@alice_mediator"
                    ]
                },
                {
                    "_id": {
                        "$oid": "6580701fd2d92bb3cd291b2b"
                    },
                    "client_did": "did:key:other",
                    "mediator_did": "did:web:alice-mediator.com:alice_mediator_pub",
                    "keylist": []
                }
            ]"##,
        )
        .unwrap()
    }

    async fn _assert_delegate_handler_err(
        err: Response,
        status: StatusCode,
        mediation_error: MediationError,
    ) {
        assert_eq!(err.status(), status);

        let body = hyper::body::to_bytes(err.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_canon::to_string(&body).unwrap(),
            json_canon::to_string(&mediation_error.json().0).unwrap()
        );
    }
}
