use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
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
        KeylistUpdateResponse, KeylistUpdateResponseBody, KeylistUpdateResult,
    },
    web::{error::MediationError, AppState, AppStateRepository},
};

pub async fn process_plain_keylist_update_message(
    state: Arc<AppState>,
    message: Message,
) -> Response {
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

            return response.into_response();
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
        .find_one_by(doc! { "client_did": sender })
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
