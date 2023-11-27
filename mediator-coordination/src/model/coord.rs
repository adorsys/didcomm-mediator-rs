use did_utils::vc::model::VerifiablePresentation;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

use super::dic::CompactDIC;

/// Types of services a mediator can offer to a registered edge agent.
/// - Inbox: Receive and store messages intended for an edge agent for eventual pickup.
/// - Outbox: Relay a message in the SSI network.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum MediatorService {
    Inbox,
    Outbox,
}

/// Message for mediation request.
///
/// It conveys key parameters as an edge agent requests mediation
/// from a cloud agent, hereinafter mediator. It includes details
/// such as the  range of services requested from the mediator and
/// a cryptographic means to ensure secure further communication
/// with the edge agent and to verify its digital signatures.
///
/// To a mediation request is expected a grant or a deny response.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediationRequest {
    /// Uniquely identifies a mediation request message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/mediate-request`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Edge agent's decentralized identifier.
    ///
    /// From this, the mediator MUST be able to derive crypto keys to
    /// enable encrypted peer communication and signature verification.
    pub did: String,

    /// Services requested from the mediator.
    pub services: BTreeSet<MediatorService>,

    /// Business-defined presentation to be verified
    /// by the mediator to avoid spamming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anti_spam: Option<VerifiablePresentation>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Message for mediation grant.
///
/// It conveys a positive response from the mediator to a mediation request,
/// carrying details the edge agent will be responsible to advertise including
/// assertions for dedicated interaction channels (DICs).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediationGrant {
    /// Uniquely identifies a mediation grant message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/mediate-grant`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Mediator's endpoint.
    pub endpoint: String,

    /// DICs (Dedicated Interaction Channels)
    ///
    /// They represent on their own a proof of authorized interaction
    /// delivered by the mediator according to an edge agent's request.
    pub dic: Vec<CompactDIC>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Message for mediation deny.
///
/// It conveys a negative response from the mediator to a mediation request.
/// This can be issued for several reasons, including business-specific ones.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediationDeny {
    /// Uniquely identifies a mediation deny message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/mediate-deny`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    use crate::constant::*;

    #[test]
    fn can_serialize_mediation_request_message() {
        let mediation_request = MediationRequest {
            id: "id_alice_mediation_request".to_string(),
            message_type: MEDIATE_REQUEST_2_0.to_string(),
            did: "did:key:alice_identity_pub@alice_mediator".to_string(),
            services: [MediatorService::Inbox, MediatorService::Outbox]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let expected = json!({
            "@id": "id_alice_mediation_request",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-request",
            "did": "did:key:alice_identity_pub@alice_mediator",
            "services": ["inbox", "outbox"]
        });

        assert_eq!(
            json_canon::to_string(&mediation_request).unwrap(),
            json_canon::to_string(&expected).unwrap(),
        )
    }

    #[test]
    fn can_deserialize_mediation_request_message() {
        let msg = r#"{
            "@id": "id_alice_mediation_request",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-request",
            "did": "did:key:alice_identity_pub@alice_mediator",
            "services": ["inbox", "outbox"],
            "antiSpam": {
                "@context": [
                    "https://www.w3.org/ns/credentials/v2"
                ],
                "type": [
                    "VerifiablePresentation"
                ],
                "id": "http://example.edu/credentials/3732",
                "holder": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                "verifiableCredential": [
                    {
                        "@context": [
                            "https://www.w3.org/ns/credentials/v2",
                            "https://www.dial.com/ns/crypto-checks/v1"
                        ],
                        "type": [
                            "VerifiableCredential",
                            "CryptoCheck"
                        ],
                        "credentialSubject": {
                            "id":"did:ethr:0xb9c5714089478a327f09197987f16f9e5d936e8a",
                            "amount": {
                                "value": 200,
                                "currency": "EUR"
                            }
                        },
                        "id": "https://www.dial.com//37325264562435234",
                        "issuer": "did:key:z6Mko9ZYgf4yzBmVSY3SpKxRsYDbqECK67T6zGMAoY5v8ikP",
                        "validFrom": "2023-03-05T19:23:24Z",
                        "validUntil": "2023-12-31T19:23:24Z"
                    }
                ]
            }
        }"#;

        // Assert deserialization

        let mediation_request: MediationRequest = serde_json::from_str(msg).unwrap();

        assert_eq!(
            mediation_request.id,
            "id_alice_mediation_request".to_string()
        );
        assert_eq!(&mediation_request.message_type, MEDIATE_REQUEST_2_0);
        assert_eq!(
            &mediation_request.did,
            "did:key:alice_identity_pub@alice_mediator"
        );
        assert_eq!(
            mediation_request.services,
            [MediatorService::Inbox, MediatorService::Outbox]
                .into_iter()
                .collect()
        );
        assert!(mediation_request.anti_spam.is_some());

        // Assert re-serialization

        assert_eq!(
            json_canon::to_string(&mediation_request).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    #[test]
    fn can_serialize_mediation_grant_message() {
        let mediation_grant = MediationGrant {
            id: "id_alice_mediation_grant".to_string(),
            message_type: MEDIATE_GRANT_2_0.to_string(),
            endpoint: "https://alice-mediator.com".to_string(),
            dic: vec![
                CompactDIC::Outbox("alice_out_opaque_dic".to_owned()),
                CompactDIC::Inbox("alice_in_opaque_dic".to_owned()),
            ],
            ..Default::default()
        };

        let expected = json!({
            "@id": "id_alice_mediation_grant",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-grant",
            "endpoint": "https://alice-mediator.com",
            "dic": ["outbox:alice_out_opaque_dic", "inbox:alice_in_opaque_dic"]
        });

        assert_eq!(
            json_canon::to_string(&mediation_grant).unwrap(),
            json_canon::to_string(&expected).unwrap(),
        )
    }

    #[test]
    fn can_deserialize_mediation_grant_message() {
        let msg = r#"{
            "@id": "id_alice_mediation_grant",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-grant",
            "endpoint": "https://alice-mediator.com",
            "dic": ["outbox:alice_out_opaque_dic", "inbox:alice_in_opaque_dic"]
        }"#;

        // Assert deserialization

        let mediation_grant: MediationGrant = serde_json::from_str(msg).unwrap();

        assert_eq!(&mediation_grant.id, "id_alice_mediation_grant");
        assert_eq!(&mediation_grant.message_type, MEDIATE_GRANT_2_0);
        assert_eq!(&mediation_grant.endpoint, "https://alice-mediator.com");
        assert_eq!(
            mediation_grant.dic,
            vec![
                CompactDIC::Outbox("alice_out_opaque_dic".to_owned()),
                CompactDIC::Inbox("alice_in_opaque_dic".to_owned())
            ]
        );

        // Assert re-serialization

        assert_eq!(
            json_canon::to_string(&mediation_grant).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    #[test]
    fn can_serialize_mediation_deny_message() {
        let mediation_deny = MediationDeny {
            id: "id_alice_mediation_deny".to_string(),
            message_type: MEDIATE_DENY_2_0.to_string(),
            ..Default::default()
        };

        let expected = json!({
            "@id": "id_alice_mediation_deny",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-deny"
        });

        assert_eq!(
            json_canon::to_string(&mediation_deny).unwrap(),
            json_canon::to_string(&expected).unwrap(),
        )
    }

    #[test]
    fn can_deserialize_mediation_deny_message() {
        let msg = r#"{
            "@id": "id_alice_mediation_deny",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-deny"
        }"#;

        // Assert deserialization
        let mediation_deny: MediationDeny = serde_json::from_str(msg).unwrap();
        assert_eq!(mediation_deny.id, "id_alice_mediation_deny".to_string());
        assert_eq!(mediation_deny.message_type, MEDIATE_DENY_2_0.to_string());

        // Assert re-serialization
        assert_eq!(
            json_canon::to_string(&mediation_deny).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }
}
