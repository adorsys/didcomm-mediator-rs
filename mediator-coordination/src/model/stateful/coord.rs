use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Message to notify the mediator of keys in use by the recipient.
///
/// See https://github.com/hyperledger/aries-rfcs/tree/main/features/0211-route-coordination#keylist-update
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdate {
    /// Uniquely identifies a keylist update message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/1.0/keylist-update`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// List of commands to update keys in use
    pub updates: Vec<KeylistUpdateCommand>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Specifies a command for keylist update
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdateCommand {
    /// Key on which to apply an action
    pub recipient_key: String,

    /// Add or remove
    pub action: KeylistUpdateAction,
}

/// Specifies an action for a keylist update command
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeylistUpdateAction {
    Add,
    Remove,
}

/// Response message to confirm requested keylist updates.
///
/// See https://github.com/hyperledger/aries-rfcs/tree/main/features/0211-route-coordination#keylist-update-response
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdateResponse {
    /// Uniquely identifies a keylist update response message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/1.0/keylist-update-response`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Confirmations to requested keylist updates.
    pub updated: Vec<KeylistUpdateConfirmation>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Conveys a result to a requested keylist update.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdateConfirmation {
    /// Key at which an action is directed
    pub recipient_key: String,

    /// Add or remove
    pub action: KeylistUpdateAction,

    /// Result confirmation
    pub result: KeylistUpdateResult,
}

/// Specifies a result to a keylist update command.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeylistUpdateResult {
    ClientError,
    ServerError,
    NoChange,
    Success,
}

/// Message to query mediator for a list of keys registered for this connection.
///
/// See https://github.com/hyperledger/aries-rfcs/tree/main/features/0211-route-coordination#key-list-query
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct KeylistQuery {
    /// Uniquely identifies a keylist query message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/1.0/keylist-query`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Optional pagination details.
    pub paginate: Option<KeylistQueryPaginate>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Pagination details for a keylist query.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistQueryPaginate {
    pub limit: i32,
    pub offset: i32,
}

/// Response to key list query, containing retrieved keys.
///
/// See https://github.com/hyperledger/aries-rfcs/tree/main/features/0211-route-coordination#key-list
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct Keylist {
    /// Uniquely identifies a keylist query response message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/1.0/keylist`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// List of retrieved keys.
    pub keys: Vec<KeylistEntry>,

    /// Optional pagination details.
    pub pagination: Option<KeylistPagination>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Keylist entry for a specific key.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistEntry {
    /// Retrieved key
    pub recipient_key: String,
}

/// Pagination details for a keylist query.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistPagination {
    pub count: i32,
    pub offset: i32,
    pub remaining: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    use crate::constant::*;

    #[test]
    fn can_serde_keylist_update_action_enum() {
        let variants = [
            (KeylistUpdateAction::Add, r#""add""#),
            (KeylistUpdateAction::Remove, r#""remove""#),
        ];

        for (variant, serialized) in variants {
            // Serialization
            assert_eq!(serde_json::to_string(&variant).unwrap(), serialized);

            // De-serialization
            assert_eq!(
                serde_json::from_str::<KeylistUpdateAction>(serialized).unwrap(),
                variant
            );
        }
    }

    #[test]
    fn can_serialize_keylist_update_message() {
        let keylist_update = KeylistUpdate {
            id: "id_alice_keylist_update_request".to_string(),
            message_type: KEYLIST_UPDATE_1_0.to_string(),
            updates: vec![KeylistUpdateCommand {
                recipient_key: String::from("did:key:alice_identity_pub1@alice_mediator"),
                action: KeylistUpdateAction::Add,
            }],
            additional_properties: None,
        };

        let expected = json!({
            "@id": "id_alice_keylist_update_request",
            "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist-update",
            "updates": [
                {
                    "recipient_key": "did:key:alice_identity_pub1@alice_mediator",
                    "action": "add",
                }
            ]
        });

        assert_eq!(
            json_canon::to_string(&keylist_update).unwrap(),
            json_canon::to_string(&expected).unwrap(),
        )
    }

    #[test]
    fn can_deserialize_keylist_update_message() {
        let msg = r#"{
            "@id": "id_alice_keylist_update_request",
            "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist-update",
            "updates": [
                {
                    "recipient_key": "did:key:alice_identity_pub1@alice_mediator",
                    "action": "add"
                }
            ]
        }"#;

        // Assert deserialization

        let keylist_update: KeylistUpdate = serde_json::from_str(msg).unwrap();

        assert_eq!(&keylist_update.id, "id_alice_keylist_update_request");
        assert_eq!(&keylist_update.message_type, KEYLIST_UPDATE_1_0);

        assert_eq!(
            keylist_update.updates,
            vec![KeylistUpdateCommand {
                recipient_key: String::from("did:key:alice_identity_pub1@alice_mediator"),
                action: KeylistUpdateAction::Add,
            }]
        );

        // Assert re-serialization

        assert_eq!(
            json_canon::to_string(&keylist_update).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    #[test]
    fn can_serde_keylist_update_result_enum() {
        let variants = [
            (KeylistUpdateResult::ClientError, r#""client_error""#),
            (KeylistUpdateResult::ServerError, r#""server_error""#),
            (KeylistUpdateResult::NoChange, r#""no_change""#),
            (KeylistUpdateResult::Success, r#""success""#),
        ];

        for (variant, serialized) in variants {
            // Serialization
            assert_eq!(serde_json::to_string(&variant).unwrap(), serialized);

            // De-serialization
            assert_eq!(
                serde_json::from_str::<KeylistUpdateResult>(serialized).unwrap(),
                variant
            );
        }
    }

    #[test]
    fn can_serialize_keylist_update_response_message() {
        let keylist_update_response = KeylistUpdateResponse {
            id: "id_alice_keylist_update_response".to_string(),
            message_type: KEYLIST_UPDATE_RESPONSE_1_0.to_string(),
            updated: vec![KeylistUpdateConfirmation {
                recipient_key: String::from("did:key:alice_identity_pub1@alice_mediator"),
                action: KeylistUpdateAction::Add,
                result: KeylistUpdateResult::Success,
            }],
            additional_properties: None,
        };

        let expected = json!({
            "@id": "id_alice_keylist_update_response",
            "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist-update-response",
            "updated": [
                {
                    "recipient_key": "did:key:alice_identity_pub1@alice_mediator",
                    "action": "add",
                    "result": "success",
                }
            ]
        });

        assert_eq!(
            json_canon::to_string(&keylist_update_response).unwrap(),
            json_canon::to_string(&expected).unwrap(),
        )
    }

    #[test]
    fn can_deserialize_keylist_update_response_message() {
        let msg = r#"{
            "@id": "id_alice_keylist_update_response",
            "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist-update-response",
            "updated": [
                {
                    "recipient_key": "did:key:alice_identity_pub1@alice_mediator",
                    "action": "add",
                    "result": "success"
                }
            ]
        }"#;

        // Assert deserialization

        let keylist_update_response: KeylistUpdateResponse = serde_json::from_str(msg).unwrap();

        assert_eq!(
            &keylist_update_response.id,
            "id_alice_keylist_update_response"
        );
        assert_eq!(
            &keylist_update_response.message_type,
            KEYLIST_UPDATE_RESPONSE_1_0
        );

        assert_eq!(
            keylist_update_response.updated,
            vec![KeylistUpdateConfirmation {
                recipient_key: String::from("did:key:alice_identity_pub1@alice_mediator"),
                action: KeylistUpdateAction::Add,
                result: KeylistUpdateResult::Success,
            }]
        );

        // Assert re-serialization

        assert_eq!(
            json_canon::to_string(&keylist_update_response).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }
}
