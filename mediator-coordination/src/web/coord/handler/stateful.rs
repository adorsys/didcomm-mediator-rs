use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson::doc;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    constant::{KEYLIST_UPDATE_2_0, KEYLIST_UPDATE_RESPONSE_2_0},
    model::stateful::coord::{
        entity::Connection, KeylistUpdate, KeylistUpdateAction, KeylistUpdateConfirmation,
        KeylistUpdateResponse, KeylistUpdateResponseBody, KeylistUpdateResult,
    },
    web::{coord::error::MediationError, AppState, AppStateRepository},
};

#[axum::debug_handler]
pub async fn process_plain_keylist_update_message(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    Json(keylist_update): Json<KeylistUpdate>,
) -> Response {
    // Initial checks

    // Temp! Read declared sender from message
    let sender = query.get("sender").cloned();

    // Validate sender
    if sender.is_none() {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::Generic(String::from("no declared sender")).json(),
        );

        return response.into_response();
    }

    // Validate message type
    if keylist_update.message_type != KEYLIST_UPDATE_2_0 {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType.json(),
        );

        return response.into_response();
    }

    // Retrieve repository to connection entities

    let AppStateRepository {
        connection_repository,
        ..
    } = state.repository.expect("missing persistence layer");

    // Find connection for this keylist update

    let connection = match connection_repository
        .find_one_by(doc! { "client_did": sender.unwrap() })
        .await
        .unwrap()
    {
        Some(connection) => connection,
        None => {
            let response = (
                StatusCode::UNAUTHORIZED,
                MediationError::UncoordinatedSender.json(),
            );

            return response.into_response();
        }
    };

    // Perform updates to persist

    let mut updated_keylist = connection.keylist.clone();
    let updates = keylist_update.body.updates;

    // Check if a specific key is duplicated across commands
    let key_is_duplicate = |recipient_did| {
        updates
            .iter()
            .filter(|e| &e.recipient_did == recipient_did)
            .count()
            > 1
    };

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

    let response = (
        StatusCode::ACCEPTED,
        Json(json!(KeylistUpdateResponse {
            id: format!("urn:uuid:{}", Uuid::new_v4()),
            message_type: KEYLIST_UPDATE_RESPONSE_2_0.to_string(),
            body: KeylistUpdateResponseBody {
                updated: confirmations
            },
            ..Default::default()
        })),
    );

    response.into_response()
}
