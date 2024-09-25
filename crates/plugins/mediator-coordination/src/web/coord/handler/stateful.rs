use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use did_utils::{
    crypto::{Ed25519KeyPair, Generate, ToMultikey, X25519KeyPair},
    didcore::Service,
    jwk::Jwk,
    methods::{DidPeer, Purpose, PurposedKey},
};
use didcomm::Message;
use mongodb::bson::{doc, oid::ObjectId};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::{
    constant::{KEYLIST_2_0, KEYLIST_UPDATE_RESPONSE_2_0, MEDIATE_DENY_2_0, MEDIATE_GRANT_2_0},
    model::stateful::{coord::{
        Keylist, KeylistBody, KeylistEntry, KeylistUpdateAction, KeylistUpdateBody,
        KeylistUpdateConfirmation, KeylistUpdateResponseBody, KeylistUpdateResult, MediationDeny,
        MediationGrant, MediationGrantBody,
    }, entity::{Connection, Secrets, VerificationMaterial}},
    web::{coord::midlw::{self, ensure_jwm_type_is_mediation_request, ensure_transport_return_route_is_decorated_all}, 
        error::MediationError, AppState, AppStateRepository},
};


/// Process a DIDComm mediate request
pub async fn process_mediate_request(
    state: &AppState,
    plain_message: &Message,
) -> Result<Message, Response> {

    
    // This is to Check message type compliance
    midlw::run!(ensure_jwm_type_is_mediation_request(&plain_message));

    // This is to Check explicit agreement to HTTP responding
    midlw::run!(ensure_transport_return_route_is_decorated_all(
        &plain_message
    ));
    
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
            id: Some(ObjectId::new()),
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
            id: Some(ObjectId::new()),
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
        body: MediationGrantBody {
            routing_did: routing_did.to_string(),
        },
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
        DidPeer::create_did_peer_2(&keys, &services).unwrap(),
        auth_keys,
        agreem_keys,
    )
}

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

pub async fn process_plain_keylist_query_message(
    state: Arc<AppState>,
    message: Message,
) -> Result<Message, Response> {
    println!("Processing keylist query...");
    let sender = message
        .from
        .expect("unpacking middleware failed to prevent anonymous senders");

    let AppStateRepository {
        connection_repository,
        ..
    } = state
        .repository
        .as_ref()
        .expect("missing persistence layer");

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

    println!("keylist: {:?}", connection);

    let keylist_entries = connection
        .keylist
        .iter()
        .map(|key| KeylistEntry {
            recipient_did: key.clone(),
        })
        .collect::<Vec<KeylistEntry>>();

    let body = KeylistBody {
        keys: keylist_entries,
        pagination: None,
    };

    let keylist_object = Keylist {
        id: format!("urn:uuid:{}", Uuid::new_v4()),
        message_type: KEYLIST_2_0.to_string(),
        body: body,
        additional_properties: None,
    };

    let mediator_did = &state.diddoc.id;

    let message = Message::build(
        format!("urn:uuid:{}", Uuid::new_v4()),
        KEYLIST_2_0.to_string(),
        json!(keylist_object),
    )
    .to(sender.clone())
    .from(mediator_did.clone())
    .finalize();

    println!("message: {:?}", message);

    Ok(message)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    use crate::{
        repository::stateful::tests::MockConnectionRepository, web::handler::tests as global,
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
    async fn test_keylist_query_success() {
        let state = setup(_initial_connections());

        // Prepare request
        let message = Message::build(
            "id_alice_keylist_query".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-query".to_owned(),
            json!({}),
        )
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Process request
        let response = process_plain_keylist_query_message(Arc::clone(&state), message)
            .await
            .unwrap();

        assert_eq!(response.type_, KEYLIST_2_0);
        assert_eq!(response.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(response.to.unwrap(), vec![global::_edge_did()]);

    }
    #[tokio::test]
    async fn test_keylist_query_malformed_request() {
        let state = setup(_initial_connections());

        // Prepare request with a sender that is not in the system 
        let message = Message::build(
            "id_alice_keylist_query".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-query".to_owned(),
            json!({}),
        )
        .to(global::_mediator_did(&state))
        .from("did:example:uncoordinated_sender".to_string()) 
        .finalize();

        // Process request
        let err = process_plain_keylist_query_message(Arc::clone(&state), message)
            .await
            .unwrap_err();
        // Assert issued error for uncoordinated sender
        _assert_delegate_handler_err(
            err,
            StatusCode::UNAUTHORIZED,
            MediationError::UncoordinatedSender,
        )
        .await;
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
                        "routing_did": "did:key:generated",
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
                        "routing_did": "did:key:generated",
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
                    "routing_did": "did:key:generated",
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
                    "routing_did": "did:key:generated",
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
                    "routing_did": "did:key:generated",
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

    use did_utils::crypto::{KeyMaterial, BYTES_LENGTH_32};

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
        let did_method = DidPeer::default();
        let did_document = did_method.expand(&did).unwrap();

        // Check that the serviceEndpoint in the DID document matches the input
        assert_eq!(
            did_document
                .service
                .unwrap()
                .first()
                .map(|s| &s.service_endpoint),
            Some(service_endpoint.to_string()).as_ref()
        );
    }
}