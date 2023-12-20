use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson::doc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    constant::{KEYLIST_UPDATE_2_0, KEYLIST_UPDATE_RESPONSE_2_0},
    model::stateful::coord::{
        entity::Connection, KeylistUpdate, KeylistUpdateAction, KeylistUpdateConfirmation,
        KeylistUpdateResponse, KeylistUpdateResponseBody, KeylistUpdateResult,
    },
    web::{coord::error::MediationError, AppState, AppStateRepository},
};

pub async fn test_connection_repository(State(state): State<AppState>) -> Json<Value> {
    let AppStateRepository {
        connection_repository,
        ..
    } = state.repository.expect("missing persistence layer");

    let connections = connection_repository.find_all().await.unwrap();
    Json(json!(connections))
}

#[axum::debug_handler]
pub async fn process_plain_keylist_update_message(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    Json(keylist_update): Json<KeylistUpdate>,
) -> Response {
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
    let confirmations: Vec<_> = keylist_update
        .body
        .updates
        .iter()
        .map(|update| KeylistUpdateConfirmation {
            recipient_did: update.recipient_did.clone(),
            action: update.action.clone(),
            result: {
                let found = connection
                    .keylist
                    .iter()
                    .position(|x| x == &update.recipient_did);

                match found {
                    Some(index) => match &update.action {
                        KeylistUpdateAction::Add => KeylistUpdateResult::NoChange,
                        KeylistUpdateAction::Remove => {
                            updated_keylist.swap_remove(index);
                            KeylistUpdateResult::Success
                        }
                        KeylistUpdateAction::Unknown(_) => KeylistUpdateResult::ClientError,
                    },
                    None => match &update.action {
                        KeylistUpdateAction::Add => {
                            updated_keylist.push(update.recipient_did.clone());
                            KeylistUpdateResult::Success
                        }
                        KeylistUpdateAction::Remove => KeylistUpdateResult::NoChange,
                        KeylistUpdateAction::Unknown(_) => KeylistUpdateResult::ClientError,
                    },
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
            .map(|confirmation| KeylistUpdateConfirmation {
                result: KeylistUpdateResult::ServerError,
                ..confirmation
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
