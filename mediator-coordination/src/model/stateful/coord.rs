use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Message for mediation request.
///
/// It conveys key parameters as an edge agent requests mediation
/// from a cloud agent, hereinafter mediator. It includes details
/// such as the  range of services requested from the mediator and
/// a cryptographic means to ensure secure further communication
/// with the edge agent and to verify its digital signatures.
///
/// To a mediation request is expected a grant or a deny response.
/// {
// "id": "123456780",
// "type": "https://didcomm.org/coordinate-mediation/2.0/mediate-request",
// "return_route": "all"
// }
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
    // Edge agent's decentralized identifier.
    //
    // From this, the mediator MUST be able to derive crypto keys to
    // enable encrypted peer communication and signature verification.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub return_route: String,
}

/// Message for mediation deny.
///
/// It conveys a negative response from the mediator to a mediation request.
/// This can be issued for several reasons, including business-specific ones.
///
/// {
// "id": "123456780",
// "type": "https://didcomm.org/coordinate-mediation/2.0/mediate-deny",
// }
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
}

// -- To evaluate:

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
            ..Default::default()
        };

        let expected = json!({
            "@id": "id_alice_mediation_request",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-request"
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
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-request"
        }"#;

        // Assert deserialization
        let mediation_request: MediationRequest = serde_json::from_str(msg).unwrap();

        // Assert re-serialization
        assert_eq!(
            json_canon::to_string(&mediation_request).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    // #[test]
    // fn can_serialize_mediation_grant_message() {
    //     let mediation_grant = MediationGrant {
    //         id: "id_alice_mediation_grant".to_string(),
    //         message_type: MEDIATE_GRANT_2_0.to_string(),
    //         endpoint: "https://alice-mediator.com".to_string(),
    //         dic: vec![
    //             CompactDIC::Outbox("alice_out_opaque_dic".to_owned()),
    //             CompactDIC::Inbox("alice_in_opaque_dic".to_owned()),
    //         ],
    //         ..Default::default()
    //     };

    //     let expected = json!({
    //         "@id": "id_alice_mediation_grant",
    //         "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-grant",
    //         "endpoint": "https://alice-mediator.com",
    //         "dic": ["outbox:alice_out_opaque_dic", "inbox:alice_in_opaque_dic"]
    //     });

    //     assert_eq!(
    //         json_canon::to_string(&mediation_grant).unwrap(),
    //         json_canon::to_string(&expected).unwrap(),
    //     )
    // }

    // #[test]
    // fn can_deserialize_mediation_grant_message() {
    //     let msg = r#"{
    //         "@id": "id_alice_mediation_grant",
    //         "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-grant",
    //         "endpoint": "https://alice-mediator.com",
    //         "dic": ["outbox:alice_out_opaque_dic", "inbox:alice_in_opaque_dic"]
    //     }"#;

    //     // Assert deserialization

    //     let mediation_grant: MediationGrant = serde_json::from_str(msg).unwrap();

    //     assert_eq!(&mediation_grant.id, "id_alice_mediation_grant");
    //     assert_eq!(&mediation_grant.message_type, MEDIATE_GRANT_2_0);
    //     assert_eq!(&mediation_grant.endpoint, "https://alice-mediator.com");
    //     assert_eq!(
    //         mediation_grant.dic,
    //         vec![
    //             CompactDIC::Outbox("alice_out_opaque_dic".to_owned()),
    //             CompactDIC::Inbox("alice_in_opaque_dic".to_owned())
    //         ]
    //     );

    //     // Assert re-serialization

    //     assert_eq!(
    //         json_canon::to_string(&mediation_grant).unwrap(),
    //         json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
    //     )
    // }

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
