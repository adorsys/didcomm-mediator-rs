use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::constant::MEDIATE_REQUEST_2_0;

#[cfg(feature = "stateless")]
use super::stateless::coord::MediationRequest as StatelessMediationRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MediationRequest {
    /// Format for stateful standard mode
    // Stateful

    /// Format for stateless mode over DICs
    #[cfg(feature = "stateless")]
    #[serde(deserialize_with = "MediationRequest::deserialize_stateless_variant")]
    Stateless(StatelessMediationRequest),
}

impl MediationRequest {
    #[cfg(feature = "stateless")]
    fn deserialize_stateless_variant<'de, D>(
        deserializer: D,
    ) -> Result<StatelessMediationRequest, D::Error>
    where
        D: Deserializer<'de>,
    {
        match StatelessMediationRequest::deserialize(deserializer)? {
            mr if mr.message_type == MEDIATE_REQUEST_2_0 => Ok(mr),
            _ => Err(Error::custom("invalid type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "stateless")]
    #[test]
    fn test_deserialize_stateless_mediation_requests() {
        let msg = r#"{
            "@id": "id_alice_mediation_request",
            "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-request",
            "did": "did:key:alice_identity_pub@alice_mediator",
            "services": ["inbox", "outbox"]
        }"#;

        let mediation_request: MediationRequest = serde_json::from_str(msg).unwrap();
        assert!(matches!(mediation_request, MediationRequest::Stateless(_)));

        let msg = r#"{
            "@id": "id_alice_mediation_request",
            "@type": "https://didcomm.org/coordinate-mediation/3.0/mediate-request",
            "did": "did:key:alice_identity_pub@alice_mediator",
            "services": ["inbox", "outbox"]
        }"#;

        let err = serde_json::from_str::<MediationRequest>(msg).unwrap_err();
        assert!(err.to_string().contains("data did not match any variant"));
    }
}
