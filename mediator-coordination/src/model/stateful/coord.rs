use serde::{Deserialize, Serialize};

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

/// Message for mediation grant.
///
/// It conveys a positive response from the mediator to a mediation request,
/// carrying details the edge agent will be responsible to advertise including
/// assertions for dedicated interaction channels (DICs).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
    pub routing_did: String,
}


// -- To evaluate:

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::{json, Value};

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

    #[test]
    fn can_serialize_mediation_grant_message() {
        let mediation_grant = MediationGrant {
            id: "id_alice_mediation_grant".to_string(),
            message_type: MEDIATE_GRANT_2_0.to_string(),
            routing_did: "routing_did".to_string(),
            ..Default::default()
        };

        let expected = json!({
            "@id": "id_alice_mediation_grant",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-grant",
            "routing_did": "routing_did",
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
            "routing_did": "routing_did"
        }"#;

        let mediation_grant: MediationGrant = serde_json::from_str(msg).unwrap();

        // Assert deserialization
        assert_eq!(&mediation_grant.id, "id_alice_mediation_grant");
        assert_eq!(&mediation_grant.message_type, MEDIATE_GRANT_2_0);
        assert_eq!(&mediation_grant.routing_did, "routing_did");

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
