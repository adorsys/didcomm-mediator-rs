use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A DIC's service level discriminates the operability of the channel.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DICServiceLevel {
    #[default]
    Gold,
    Silver,
    Bronze,
}

/// Record of a dedicated interaction channel (DIC) to be asserted.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DICPayload {
    /// Subject's DID
    #[serde(rename = "sub")]
    pub subject: String,

    /// Issuer's DID
    #[serde(rename = "iss")]
    pub issuer: String,

    /// Service level
    #[serde(rename = "sl")]
    pub service_level: DICServiceLevel,

    /// Nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Record of a delegate dedicated interaction channel (DDIC) to be asserted.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DDICPayload {
    /// Subject's DID -- To whom the DIC is delegated
    #[serde(rename = "sub")]
    pub subject: String,

    /// Original Subject's DID
    #[serde(rename = "dic-sub")]
    pub dic_subject: String,

    /// Issuer's DID
    #[serde(rename = "iss")]
    pub issuer: String,

    /// Nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_serde_dic_service_level() {
        let entries = [
            (DICServiceLevel::Gold, r#""gold""#),
            (DICServiceLevel::Silver, r#""silver""#),
            (DICServiceLevel::Bronze, r#""bronze""#),
        ];

        for (level, level_str) in entries {
            // Serialization
            assert_eq!(serde_json::to_string(&level).unwrap(), level_str,);

            // Deserialization
            assert_eq!(
                serde_json::from_str::<DICServiceLevel>(&level_str).unwrap(),
                level,
            );
        }
    }

    #[test]
    fn can_serde_dic_payload() {
        let msg = r#"{
            "sub": "did:key:alice_identity_pub@alice_mediator",
            "iss": "did:web:alice-mediator.com:alice_mediator_pub",
            "sl": "gold",
            "nonce": "43f84868-0632-4471-b6dd-d63fa12c21f6"
        }"#;

        // Assert deserialization
        let dic_payload: DICPayload = serde_json::from_str(msg).unwrap();
        assert_eq!(
            dic_payload.subject,
            "did:key:alice_identity_pub@alice_mediator"
        );
        assert_eq!(
            dic_payload.issuer,
            "did:web:alice-mediator.com:alice_mediator_pub"
        );
        assert_eq!(dic_payload.service_level, DICServiceLevel::Gold);
        assert_eq!(
            dic_payload.nonce.as_deref().unwrap(),
            "43f84868-0632-4471-b6dd-d63fa12c21f6"
        );

        // Assert re-serialization
        assert_eq!(
            json_canon::to_string(&dic_payload).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    #[test]
    fn can_serde_ddic_payload() {
        let msg = r#"{
            "sub": "did:key:bob_identity_pub@alice",
            "iss": "did:web:alice-mediator.com:alice_mediator_pub",
            "dic-sub": "did:key:alice_identity_pub@alice_mediator",
            "nonce": "43f84868-0632-4471-b6dd-d63fa12c21f6"
        }"#;

        // Assert deserialization
        let ddic_payload: DDICPayload = serde_json::from_str(msg).unwrap();
        assert_eq!(ddic_payload.subject, "did:key:bob_identity_pub@alice");
        assert_eq!(
            ddic_payload.dic_subject,
            "did:key:alice_identity_pub@alice_mediator"
        );
        assert_eq!(
            ddic_payload.issuer,
            "did:web:alice-mediator.com:alice_mediator_pub"
        );
        assert_eq!(
            ddic_payload.nonce.as_deref().unwrap(),
            "43f84868-0632-4471-b6dd-d63fa12c21f6"
        );

        // Assert re-serialization
        assert_eq!(
            json_canon::to_string(&ddic_payload).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }
}
