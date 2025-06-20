use crate::{
    constants::{KEYLIST_2_0, KEYLIST_UPDATE_RESPONSE_2_0, MEDIATE_DENY_2_0, MEDIATE_GRANT_2_0},
    errors::MediationError,
    handler::midlw::ensure_jwm_type_is_mediation_request,
    model::stateful::coord::{
        KeylistBody, KeylistEntry, KeylistUpdateAction, KeylistUpdateBody,
        KeylistUpdateConfirmation, KeylistUpdateResponseBody, KeylistUpdateResult,
        MediationGrantBody,
    },
};

use did_utils::{
    crypto::{Ed25519KeyPair, Generate, ToMultikey, X25519KeyPair},
    didcore::Service,
    jwk::Jwk,
    methods::{DidPeer, Purpose, PurposedKey},
};
use didcomm::{
    did::{DIDDoc, DIDResolver},
    Message,
};
use mongodb::bson::doc;
use serde_json::{json, Value};
use shared::{
    breaker::Error as BreakerError,
    midlw::ensure_transport_return_route_is_decorated_all,
    repository::entity::Connection,
    state::{AppState, AppStateRepository},
};
use std::sync::Arc;
use uuid::Uuid;

/// Process a DIDComm mediate request
pub(crate) async fn process_mediate_request(
    state: Arc<AppState>,
    plain_message: Message,
) -> Result<Option<Message>, MediationError> {
    // Check if the circuit breaker is open
    check_circuit_breaker(&state)?;

    // Validate message type and return route
    ensure_jwm_type_is_mediation_request(&plain_message)?;
    ensure_transport_return_route_is_decorated_all(&plain_message)
        .map_err(|_| MediationError::NoReturnRouteAllDecoration)?;

    let mediator_did = &state.diddoc.id;
    let sender_did = sender_did(&plain_message)?;

    let repository = state
        .repository
        .as_ref()
        .ok_or(MediationError::InternalServerError)?;

    // Check existing mediation
    let existing_connection = state
        .db_circuit_breaker
        .call(|| {
            repository
                .connection_repository
                .find_one_by(doc! { "client_did": sender_did})
        })
        .await
        .map_err(|err| match err {
            BreakerError::CircuitOpen => MediationError::ServiceUnavailable,
            _ => MediationError::InternalServerError,
        })?;

    if existing_connection.is_some() {
        tracing::info!("Sending mediate deny.");
        return Ok(Some(
            Message::build(
                format!("urn:uuid:{}", Uuid::new_v4()),
                MEDIATE_DENY_2_0.to_string(),
                json!(Value::Null),
            )
            .to(sender_did.to_owned())
            .from(mediator_did.clone())
            .finalize(),
        ));
    }

    // Process mediation grant
    process_mediation_grant(&state, mediator_did, sender_did).await
}

async fn process_mediation_grant(
    state: &Arc<AppState>,
    mediator_did: &str,
    sender_did: &str,
) -> Result<Option<Message>, MediationError> {
    tracing::info!("Sending mediate grant.");
    let (routing_did, auth_keys, agreem_keys) = generate_did_peer(state.public_domain.to_string());

    let repository = state
        .repository
        .as_ref()
        .ok_or(MediationError::InternalServerError)?;

    let diddoc = state
        .did_resolver
        .resolve(&routing_did)
        .await
        .map_err(|err| {
            tracing::error!("Failed to resolve DID: {:?}", err);
            MediationError::InternalServerError
        })?
        .ok_or(MediationError::InternalServerError)?;

    // Store keys
    store_keys(state, repository, &diddoc, auth_keys, agreem_keys).await?;

    let mediation_grant = create_mediation_grant(&routing_did);

    let new_connection = Connection {
        id: None,
        client_did: sender_did.to_string(),
        mediator_did: mediator_did.to_string(),
        keylist: vec![],
        routing_did,
    };

    // Store connection
    state
        .db_circuit_breaker
        .call(|| {
            repository
                .connection_repository
                .store(new_connection.to_owned())
        })
        .await
        .map_err(|err| match err {
            BreakerError::CircuitOpen => MediationError::ServiceUnavailable,
            BreakerError::Inner(err) => {
                tracing::error!("Failed to store connection: {err:?}");
                MediationError::InternalServerError
            }
        })?;

    Ok(Some(
        Message::build(
            format!("urn:uuid:{}", Uuid::new_v4()),
            MEDIATE_GRANT_2_0.to_string(),
            json!(mediation_grant),
        )
        .to(sender_did.to_owned())
        .from(mediator_did.to_owned())
        .finalize(),
    ))
}

async fn store_keys(
    state: &Arc<AppState>,
    repository: &AppStateRepository,
    diddoc: &DIDDoc,
    auth_keys: Ed25519KeyPair,
    agreem_keys: X25519KeyPair,
) -> Result<(), MediationError> {
    let agreem_id = diddoc.key_agreement.first().unwrap();
    let agreem_keys_jwk: Jwk = agreem_keys.try_into().unwrap();

    state
        .db_circuit_breaker
        .call(|| repository.keystore.store(agreem_id, &agreem_keys_jwk))
        .await
        .map_err(|err| match err {
            BreakerError::CircuitOpen => MediationError::ServiceUnavailable,
            BreakerError::Inner(err) => {
                tracing::error!("Failed to store agreem keys: {err:?}");
                MediationError::InternalServerError
            }
        })?;

    let auth_id = diddoc.authentication.first().unwrap();
    let auth_keys_jwk: Jwk = auth_keys.try_into().unwrap();

    state
        .db_circuit_breaker
        .call(|| repository.keystore.store(auth_id, &auth_keys_jwk))
        .await
        .map_err(|err| match err {
            BreakerError::CircuitOpen => MediationError::ServiceUnavailable,
            BreakerError::Inner(err) => {
                tracing::error!("Failed to store auth keys: {err:?}");
                MediationError::InternalServerError
            }
        })?;

    Ok(())
}

#[inline]
fn create_mediation_grant(routing_did: &str) -> MediationGrantBody {
    MediationGrantBody {
        routing_did: routing_did.to_string(),
    }
}

fn generate_did_peer(service_endpoint: String) -> (String, Ed25519KeyPair, X25519KeyPair) {
    // Generate keys
    let auth_keys = Ed25519KeyPair::new().unwrap();
    let agreem_keys = X25519KeyPair::new().unwrap();

    // Format them for did:peer
    let keys = vec![
        PurposedKey {
            purpose: Purpose::Verification,
            public_key_multibase: auth_keys.to_multikey(),
        },
        PurposedKey {
            purpose: Purpose::Encryption,
            public_key_multibase: agreem_keys.to_multikey(),
        },
    ];

    // Generate service
    let services = vec![Service {
        id: String::from("#didcomm"),
        service_type: String::from("DIDCommMessaging"),
        service_endpoint: json!({
            "uri": service_endpoint,
            "accept": vec!["didcomm/v2"],
            "routingKeys": Vec::<String>::new()}),
        ..Default::default()
    }];

    (
        DidPeer::create_did_peer_2(&keys, &services).unwrap(),
        auth_keys,
        agreem_keys,
    )
}

pub(crate) async fn process_plain_keylist_update_message(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, MediationError> {
    // Circuit breaker check
    check_circuit_breaker(&state)?;

    let sender_did = sender_did(&message)?;

    let keylist_update_body: KeylistUpdateBody = serde_json::from_value(message.body.clone())
        .map_err(|_| MediationError::UnexpectedMessageFormat)?;

    let repository = state
        .repository
        .as_ref()
        .ok_or(MediationError::InternalServerError)?;

    // Find connection
    let connection = state
        .db_circuit_breaker
        .call(|| {
            repository
                .connection_repository
                .find_one_by(doc! { "client_did": sender_did })
        })
        .await
        .map_err(|err| match err {
            BreakerError::CircuitOpen => MediationError::ServiceUnavailable,
            BreakerError::Inner(err) => {
                tracing::error!("Failed to find connection: {err:?}");
                MediationError::InternalServerError
            }
        })?
        .ok_or(MediationError::UncoordinatedSender)?;

    let mut updated_keylist = connection.keylist.clone();
    let updates = keylist_update_body.updates;

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

    // Update connection
    let confirmations = state
        .db_circuit_breaker
        .call(|| {
            repository.connection_repository.update(Connection {
                keylist: updated_keylist.clone(),
                ..connection.clone()
            })
        })
        .await
        .map_err(|err| match err {
            BreakerError::CircuitOpen => MediationError::ServiceUnavailable,
            BreakerError::Inner(err) => {
                tracing::error!("Failed to update connection: {err:?}");
                MediationError::InternalServerError
            }
        })
        .map_or_else(
            |_| {
                confirmations
                    .iter()
                    .map(|confirmation| {
                        if confirmation.result != KeylistUpdateResult::ClientError {
                            KeylistUpdateConfirmation {
                                recipient_did: confirmation.recipient_did.clone(),
                                action: confirmation.action.clone(),
                                result: KeylistUpdateResult::ServerError,
                            }
                        } else {
                            confirmation.clone()
                        }
                    })
                    .collect()
            },
            |_| confirmations.clone(),
        );

    let mediator_did = &state.diddoc.id;

    Ok(Some(
        Message::build(
            format!("urn:uuid:{}", Uuid::new_v4()),
            KEYLIST_UPDATE_RESPONSE_2_0.to_string(),
            json!(KeylistUpdateResponseBody {
                updated: confirmations
            }),
        )
        .to(sender_did.to_owned())
        .from(mediator_did.to_owned())
        .finalize(),
    ))
}

pub(crate) async fn process_plain_keylist_query_message(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, MediationError> {
    // Circuit breaker check
    check_circuit_breaker(&state)?;

    let sender_did = sender_did(&message)?;

    let repository = state
        .repository
        .as_ref()
        .ok_or(MediationError::InternalServerError)?;

    // Find connection
    let connection = state
        .db_circuit_breaker
        .call(|| {
            repository
                .connection_repository
                .find_one_by(doc! { "client_did": sender_did })
        })
        .await
        .map_err(|err| match err {
            BreakerError::CircuitOpen => MediationError::ServiceUnavailable,
            BreakerError::Inner(err) => {
                tracing::error!("Failed to find connection: {err:?}");
                MediationError::InternalServerError
            }
        })?
        .ok_or(MediationError::UncoordinatedSender)?;

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

    let mediator_did = &state.diddoc.id;

    let message = Message::build(
        format!("urn:uuid:{}", Uuid::new_v4()),
        KEYLIST_2_0.to_string(),
        json!(body),
    )
    .to(sender_did.to_owned())
    .from(mediator_did.clone())
    .finalize();

    Ok(Some(message))
}

#[inline]
fn check_circuit_breaker(state: &Arc<AppState>) -> Result<(), MediationError> {
    state
        .db_circuit_breaker
        .should_allow_call()
        .then_some(())
        .ok_or(MediationError::ServiceUnavailable)
}

#[inline]
fn sender_did(message: &Message) -> Result<&str, MediationError> {
    message
        .from
        .as_deref()
        .ok_or(MediationError::MissingSenderDID)
}

#[cfg(test)]
mod tests {
    use super::*;

    use shared::{
        repository::tests::MockConnectionRepository, utils::tests_utils::tests as global,
    };

    #[allow(clippy::needless_update)]
    fn setup(initial_connections: Vec<Connection>) -> Arc<AppState> {
        let state = global::setup();

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
        let message = process_plain_keylist_query_message(Arc::clone(&state), message)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(message.type_, KEYLIST_2_0);
        assert_eq!(message.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(message.to.unwrap(), vec![global::_edge_did()]);
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
        assert_eq!(err, MediationError::UncoordinatedSender,);
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
            .unwrap()
            .expect("Response should not be None");
        let response = response;

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
            .unwrap()
            .expect("Response should not be None");
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
            .unwrap()
            .expect("Response should not be None");
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
            .unwrap()
            .expect("Response should not be None");

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
        assert_eq!(err, MediationError::UnexpectedMessageFormat,);
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
        assert_eq!(err, MediationError::UncoordinatedSender,);
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
        let expected_service_endpoint = json!({"uri": service_endpoint, "accept": vec!["didcomm/v2"], "routingKeys": Vec::<String>::new()});

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
            Some(&expected_service_endpoint)
        );
    }
}
