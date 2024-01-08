use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use did_utils::{
    crypto::{ed25519::Ed25519KeyPair, traits::Generate, x25519::X25519KeyPair},
    didcore::Service,
    key_jwk::jwk::Jwk,
    methods::{
        common::ToMultikey,
        did_peer::method::{Purpose, PurposedKey},
    },
};

use did_utils::methods::did_peer::DIDPeerMethod;

use didcomm::Message;
use mongodb::bson::{doc, oid::ObjectId};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    constant::{
        KEYLIST_UPDATE_2_0, KEYLIST_UPDATE_RESPONSE_2_0, MEDIATE_DENY_2_0, MEDIATE_GRANT_2_0,
    },
    model::stateful::coord::{
        entity::{Connection, Secrets, VerificationMaterial},
        KeylistUpdate, KeylistUpdateAction, KeylistUpdateConfirmation, KeylistUpdateResponse,
        KeylistUpdateResponseBody, KeylistUpdateResult, MediationDeny, MediationGrant,
    },
    web::{coord::error::MediationError, AppState, AppStateRepository},
};

/// Process a DIDComm mediate request
pub async fn process_mediate_request(
    state: &AppState,
    plain_message: &Message,
) -> Result<Message, Response> {
    let mediator_did = &state.diddoc.id;

    let sender_did = plain_message
        .from
        .as_ref()
        .expect("should not panic as anonymous requests are rejected earlier");

    // Retrieve repository to connection entities

    let AppStateRepository {
        connection_repository,
        ..
    } = state
        .repository
        .as_ref()
        .expect("missing persistence layer");

    // If there is already mediation, send mediate deny
    if let Some(_connection) = connection_repository
        .find_one_by(doc! { "client_did": sender_did})
        .await
        .unwrap()
    {
        println!("Sending mediate deny.");
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
        println!("Sending mediate grant.");
        // Create routing, store it and send mediation grant
        let (routing_did, auth_keys, agreem_keys) =
            generate_did_peer(state.public_domain.to_string());

        let AppStateRepository {
            secret_repository, ..
        } = state
            .repository
            .as_ref()
            .expect("missing persistence layer");

        let agreem_keys_jwk: Jwk = agreem_keys.try_into().expect("MediateRequestError");

        let agreem_keys_secret = Secrets {
            id: ObjectId::new(),
            kid: routing_did.clone(),
            type_: 1,
            verification_material: VerificationMaterial {
                format: 1,
                value: serde_json::to_value(agreem_keys_jwk).unwrap().to_string(),
            },
        };

        match secret_repository.store(agreem_keys_secret).await {
            Ok(_stored_connection) => {
                println!("Successfully stored connection.")
            }
            Err(error) => eprintln!("Error storing connection: {:?}", error),
        }

        let auth_keys_jwk: Jwk = auth_keys.try_into().expect("MediateRequestError");

        let auth_keys_secret = Secrets {
            id: ObjectId::new(),
            kid: routing_did.clone(),
            type_: 1,
            verification_material: VerificationMaterial {
                format: 1,
                value: serde_json::to_value(auth_keys_jwk).unwrap().to_string(),
            },
        };

        match secret_repository.store(auth_keys_secret).await {
            Ok(_stored_connection) => {
                println!("Successfully stored connection.")
            }
            Err(error) => eprintln!("Error storing connection: {:?}", error),
        }

        let mediation_grant = create_mediation_grant(&routing_did);

        let new_connection = Connection {
            id: None,
            client_did: sender_did.to_string(),
            mediator_did: mediator_did.to_string(),
            keylist: vec!["".to_string()],
            routing_did: routing_did,
        };

        // Use store_one to store the sample connection
        match connection_repository.store(new_connection).await {
            Ok(_stored_connection) => {
                println!("Successfully stored connection: ")
            }
            Err(error) => eprintln!("Error storing connection: {:?}", error),
        }

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

fn create_mediation_grant(routing_did: &str) -> MediationGrant {
    MediationGrant {
        id: format!("urn:uuid:{}", Uuid::new_v4()),
        message_type: MEDIATE_GRANT_2_0.to_string(),
        routing_did: routing_did.to_string(),
        ..Default::default()
    }
}

fn generate_did_peer(service_endpoint: String) -> (String, Ed25519KeyPair, X25519KeyPair) {
    // Generate keys
    let auth_keys = Ed25519KeyPair::new().unwrap();
    let agreem_keys = X25519KeyPair::new().unwrap();

    // Format them for did:peer
    let keys = vec![
        PurposedKey {
            purpose: Purpose::Encryption,
            public_key_multibase: auth_keys.to_multikey(),
        },
        PurposedKey {
            purpose: Purpose::Verification,
            public_key_multibase: agreem_keys.to_multikey(),
        },
    ];

    // Generate service
    let mut additional_properties = HashMap::new();
    additional_properties.insert("accept".to_string(), json!(["didcomm/v2"]));

    let services = vec![Service {
        id: String::from("#didcomm"),
        service_type: String::from("DIDCommMessaging"),
        service_endpoint: service_endpoint,
        additional_properties: Some(additional_properties),
    }];

    (
        DIDPeerMethod::create_did_peer_2(&keys, &services).unwrap(),
        auth_keys,
        agreem_keys,
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use did_utils::crypto::traits::{KeyMaterial, BYTES_LENGTH_32};

    #[test]
    fn test_generate_did_peer() {
        // Positive Test
        let (did_peer, auth_keys, agreem_keys) = generate_did_peer("example_endpoint".to_string());

        // Check if the generated DID Peer is not empty
        assert!(!did_peer.is_empty());

        // Check if auth_keys and agreem_keys have the right size
        assert_eq!(
            agreem_keys.public_key_bytes().unwrap().len(),
            BYTES_LENGTH_32
        );
        assert_eq!(
            agreem_keys.private_key_bytes().unwrap().len(),
            BYTES_LENGTH_32
        );
        assert_eq!(auth_keys.public_key_bytes().unwrap().len(), BYTES_LENGTH_32);
        assert_eq!(
            auth_keys.private_key_bytes().unwrap().len(),
            BYTES_LENGTH_32
        );
    }

    #[test]
    fn test_generate_did_peer_and_expand() {
        // Generate a did:peer address with a service endpoint
        let service_endpoint = "http://example.com/didcomm";
        let (did, _, _) = generate_did_peer(service_endpoint.to_string());

        // Expand the generated did:peer address to a DID document
        let did_method = DIDPeerMethod::default();
        let did_document = did_method.expand(&did).unwrap();

        // Check that the serviceEndpoint in the DID document matches the input
        assert_eq!(
            did_document.service.unwrap().first().map(|s| &s.service_endpoint),
            Some(service_endpoint.to_string()).as_ref()
        );
    }
}