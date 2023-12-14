//! Types for Coordinate Mediation v2.0
//! See https://didcomm.org/coordinate-mediation/2.0

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Header for Transports Return Route Extension
///
/// See https://github.com/hyperledger/aries-rfcs/tree/main/features/0092-transport-return-route
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReturnRouteHeader {
    None,
    Thread,
    All,
}

/// Message to notify the mediator of keys in use by the recipient.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdate {
    /// Uniquely identifies a keylist update message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/keylist-update`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Message body
    pub body: KeylistUpdateBody,

    /// Return route header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_route: Option<ReturnRouteHeader>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdateBody {
    /// List of commands to update keys in use
    pub updates: Vec<KeylistUpdateCommand>,
}

/// Specifies a command for keylist update
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdateCommand {
    /// DID on which to apply an action
    pub recipient_did: String,

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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdateResponse {
    /// Uniquely identifies a keylist update response message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/keylist-update-response`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Message body
    pub body: KeylistUpdateResponseBody,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdateResponseBody {
    /// Confirmations to requested keylist updates.
    pub updated: Vec<KeylistUpdateConfirmation>,
}

/// Conveys a result to a requested keylist update.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistUpdateConfirmation {
    /// DID at which an action was directed
    pub recipient_did: String,

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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct KeylistQuery {
    /// Uniquely identifies a keylist query message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/keylist-query`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Message body
    pub body: KeylistQueryBody,

    /// Return route header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_route: Option<ReturnRouteHeader>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistQueryBody {
    /// Optional pagination details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paginate: Option<KeylistQueryPaginate>,
}

/// Pagination details for a keylist query.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistQueryPaginate {
    pub limit: i32,
    pub offset: i32,
}

/// Response to key list query, containing retrieved keys.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct Keylist {
    /// Uniquely identifies a keylist query response message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/keylist`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Message body
    pub body: KeylistBody,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistBody {
    /// List of retrieved keys.
    pub keys: Vec<KeylistEntry>,

    /// Optional pagination details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<KeylistPagination>,
}

/// Keylist entry for a specific key.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KeylistEntry {
    /// Registered DID
    pub recipient_did: String,
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
    fn can_serde_return_route_header_enum() {
        let variants = [
            (ReturnRouteHeader::None, r#""none""#),
            (ReturnRouteHeader::Thread, r#""thread""#),
            (ReturnRouteHeader::All, r#""all""#),
        ];

        for (variant, serialized) in variants {
            // Serialization
            assert_eq!(serde_json::to_string(&variant).unwrap(), serialized);

            // De-serialization
            assert_eq!(
                serde_json::from_str::<ReturnRouteHeader>(serialized).unwrap(),
                variant
            );
        }
    }

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
    fn can_serialize_keylist_update_message() {
        let keylist_update = KeylistUpdate {
            id: "id_alice_keylist_update_request".to_string(),
            message_type: KEYLIST_UPDATE_2_0.to_string(),
            body: KeylistUpdateBody {
                updates: vec![KeylistUpdateCommand {
                    recipient_did: String::from("did:key:alice_identity_pub1@alice_mediator"),
                    action: KeylistUpdateAction::Add,
                }],
            },
            return_route: Some(ReturnRouteHeader::All),
            additional_properties: None,
        };

        let expected = json!({
            "@id": "id_alice_keylist_update_request",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/keylist-update",
            "body": {
                "updates": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                        "action": "add",
                    }
                ]
            },
            "return_route": "all",
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
            "@type": "https://didcomm.org/coordinate-mediation/2.0/keylist-update",
            "body": {
                "updates": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                        "action": "add"
                    }
                ]
            },
            "return_route": "all"
        }"#;

        // Assert deserialization

        let keylist_update: KeylistUpdate = serde_json::from_str(msg).unwrap();

        assert_eq!(&keylist_update.id, "id_alice_keylist_update_request");
        assert_eq!(&keylist_update.message_type, KEYLIST_UPDATE_2_0);
        assert_eq!(keylist_update.return_route, Some(ReturnRouteHeader::All));

        assert_eq!(
            keylist_update.body,
            KeylistUpdateBody {
                updates: vec![KeylistUpdateCommand {
                    recipient_did: String::from("did:key:alice_identity_pub1@alice_mediator"),
                    action: KeylistUpdateAction::Add,
                }]
            }
        );

        // Assert re-serialization

        assert_eq!(
            json_canon::to_string(&keylist_update).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    #[test]
    fn can_serialize_keylist_update_response_message() {
        let keylist_update_response = KeylistUpdateResponse {
            id: "id_alice_keylist_update_response".to_string(),
            message_type: KEYLIST_UPDATE_RESPONSE_2_0.to_string(),
            body: KeylistUpdateResponseBody {
                updated: vec![KeylistUpdateConfirmation {
                    recipient_did: String::from("did:key:alice_identity_pub1@alice_mediator"),
                    action: KeylistUpdateAction::Add,
                    result: KeylistUpdateResult::Success,
                }],
            },
            additional_properties: None,
        };

        let expected = json!({
            "@id": "id_alice_keylist_update_response",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/keylist-update-response",
            "body": {
            "updated": [
                {
                    "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                    "action": "add",
                    "result": "success",
                }
            ]}
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
            "@type": "https://didcomm.org/coordinate-mediation/2.0/keylist-update-response",
            "body": {
                "updated": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                        "action": "add",
                        "result": "success"
                    }
                ]
            }
        }"#;

        // Assert deserialization

        let keylist_update_response: KeylistUpdateResponse = serde_json::from_str(msg).unwrap();

        assert_eq!(
            &keylist_update_response.id,
            "id_alice_keylist_update_response"
        );
        assert_eq!(
            &keylist_update_response.message_type,
            KEYLIST_UPDATE_RESPONSE_2_0
        );

        assert_eq!(
            keylist_update_response.body,
            KeylistUpdateResponseBody {
                updated: vec![KeylistUpdateConfirmation {
                    recipient_did: String::from("did:key:alice_identity_pub1@alice_mediator"),
                    action: KeylistUpdateAction::Add,
                    result: KeylistUpdateResult::Success,
                }]
            }
        );

        // Assert re-serialization

        assert_eq!(
            json_canon::to_string(&keylist_update_response).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    #[test]
    fn can_serialize_keylist_query_message() {
        let keylist_update = KeylistQuery {
            id: "id_alice_keylist_query".to_string(),
            message_type: KEYLIST_QUERY_2_0.to_string(),
            body: KeylistQueryBody {
                paginate: Some(KeylistQueryPaginate {
                    limit: 30,
                    offset: 0,
                }),
            },
            return_route: Some(ReturnRouteHeader::All),
            additional_properties: None,
        };

        let expected = json!({
            "@id": "id_alice_keylist_query",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/keylist-query",
            "body": {
                "paginate": {
                    "limit": 30,
                    "offset": 0,
                }
            },
            "return_route": "all",
        });

        assert_eq!(
            json_canon::to_string(&keylist_update).unwrap(),
            json_canon::to_string(&expected).unwrap(),
        )
    }

    #[test]
    fn can_deserialize_keylist_query_message() {
        let msg = r#"{
            "@id": "id_alice_keylist_query",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/keylist-query",
            "body": {
                "paginate": {
                    "limit": 30,
                    "offset": 0
                }
            },
            "return_route": "all"
        }"#;

        // Assert deserialization

        let keylist_query: KeylistQuery = serde_json::from_str(msg).unwrap();

        assert_eq!(&keylist_query.id, "id_alice_keylist_query");
        assert_eq!(&keylist_query.message_type, KEYLIST_QUERY_2_0);
        assert_eq!(keylist_query.return_route, Some(ReturnRouteHeader::All));

        assert_eq!(
            keylist_query.body,
            KeylistQueryBody {
                paginate: Some(KeylistQueryPaginate {
                    limit: 30,
                    offset: 0
                })
            }
        );

        // Assert re-serialization

        assert_eq!(
            json_canon::to_string(&keylist_query).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    #[test]
    fn can_serialize_keylist_message() {
        let keylist = Keylist {
            id: "id_alice_keylist".to_string(),
            message_type: KEYLIST_2_0.to_string(),
            body: KeylistBody {
                keys: vec![KeylistEntry {
                    recipient_did: String::from("did:key:alice_identity_pub1@alice_mediator"),
                }],
                pagination: Some(KeylistPagination {
                    count: 30,
                    offset: 30,
                    remaining: 100,
                }),
            },
            additional_properties: None,
        };

        let expected = json!({
            "@id": "id_alice_keylist",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/keylist",
            "body": {
                "keys": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    }
                ],
                "pagination": {
                    "count": 30,
                    "offset": 30,
                    "remaining": 100,
                }
            }
        });

        assert_eq!(
            json_canon::to_string(&keylist).unwrap(),
            json_canon::to_string(&expected).unwrap(),
        )
    }

    #[test]
    fn can_deserialize_keylist_message() {
        let msg = r#"{
            "@id": "id_alice_keylist",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/keylist",
            "body": {
                "keys": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    }
                ],
                "pagination": {
                    "count": 30,
                    "offset": 30,
                    "remaining": 100
                }
            }
        }"#;

        // Assert deserialization

        let keylist: Keylist = serde_json::from_str(msg).unwrap();

        assert_eq!(&keylist.id, "id_alice_keylist");
        assert_eq!(&keylist.message_type, KEYLIST_2_0);

        assert_eq!(
            keylist.body,
            KeylistBody {
                keys: vec![KeylistEntry {
                    recipient_did: String::from("did:key:alice_identity_pub1@alice_mediator"),
                }],
                pagination: Some(KeylistPagination {
                    count: 30,
                    offset: 30,
                    remaining: 100
                })
            }
        );

        // Assert re-serialization

        assert_eq!(
            json_canon::to_string(&keylist).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }
}
