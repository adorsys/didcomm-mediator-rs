use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use did_utils::methods::did_key::DIDKeyMethod;
use didcomm::Message;
use mongodb::bson::doc;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    constant::{
        KEYLIST_UPDATE_2_0, KEYLIST_UPDATE_RESPONSE_2_0, MEDIATE_DENY_2_0, MEDIATE_GRANT_2_0,
    },
    model::stateful::coord::{
        entity::Connection, KeylistUpdate, KeylistUpdateAction, KeylistUpdateConfirmation,
        KeylistUpdateResponse, KeylistUpdateResponseBody, KeylistUpdateResult, MediationDeny,
        MediationGrant, MediationRequest,
    },
    web::{coord::error::MediationError, AppState, AppStateRepository},
};

const IS_THERE_EXISTING_CONNECTION: bool = false;

/// Process a DIC-wise mediation request
pub async fn process_mediate_request(
    state: &AppState,
    plain_message: &Message,
    mediation_request: &MediationRequest,
) -> Result<Message, Response> {
    let mediator_did = &state.diddoc.id;
    println!("mediation_request: {:#?}", mediation_request);

    let sender_did = plain_message
        .from
        .as_ref()
        .expect("should not panic as anonymous requests are rejected earlier");

    // This will be replaced by a proper DB check
    // If there is already mediation, send denial
    if IS_THERE_EXISTING_CONNECTION {
        return Ok(Message::build(
            format!("urn:uuid:{}", Uuid::new_v4()),
            MEDIATE_DENY_2_0.to_string(),
            json!(MediationDeny {
                id: format!("urn:uuid:{}", Uuid::new_v4()),
                message_type: MEDIATE_DENY_2_0.to_string(),
                ..Default::default()
            }),
        )
        .to(sender_did.clone())
        .from(mediator_did.clone())
        .finalize());
    } else {
        /* Issue mediate grant response */

        // Create routing, store it and send mediation grant
        let did = DIDKeyMethod::generate();

        let mediation_grant = MediationGrant {
            id: format!("urn:uuid:{}", Uuid::new_v4()),
            message_type: MEDIATE_GRANT_2_0.to_string(),
            routing_did: did.unwrap().to_string(),
            ..Default::default()
        };

        //TO-DO: Store it

        Ok(Message::build(
            format!("urn:uuid:{}", Uuid::new_v4()),
            mediation_grant.message_type.clone(),
            json!(mediation_grant),
        )
        .to(sender_did.clone())
        .from(mediator_did.clone())
        .finalize())
    }
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

    // Prepare handles to relevant collections

    let mut updated_keylist = connection.keylist.clone();
    let updates = keylist_update.body.updates;

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
