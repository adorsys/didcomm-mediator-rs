use did_utils::jwk::Jwk;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::jose::jws::{self, JwsAlg, JwsError, JwsHeader};

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

/// Can sign payloads into JWTs
pub trait JwtAssertable: Serialize {
    /// Sign into JWT
    fn sign(&self, jwk: &Jwk, kid: Option<String>) -> Result<String, JwsError>;

    /// Sign into JWT with custom `typ` header
    fn sign_with_typ(&self, typ: &str, jwk: &Jwk, kid: Option<String>) -> Result<String, JwsError> {
        let header = JwsHeader {
            typ: Some(typ.to_string()),
            kid,
            alg: JwsAlg::EdDSA,
            ..Default::default()
        };

        jws::make_compact_jws(&header, json!(self), jwk)
    }
}

#[allow(unused)]
impl JwtAssertable for DICPayload {
    fn sign(&self, jwk: &Jwk, kid: Option<String>) -> Result<String, JwsError> {
        self.sign_with_typ("dic/v001", jwk, kid)
    }
}

#[allow(unused)]
impl JwtAssertable for DDICPayload {
    fn sign(&self, jwk: &Jwk, kid: Option<String>) -> Result<String, JwsError> {
        self.sign_with_typ("ddic/v001", jwk, kid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{self, MockFileSystem};
    use did_utils::crypto::ToPublic;

    fn setup() -> Jwk {
        let mut mock_fs = MockFileSystem;

        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let (_, pubkey) = util::extract_assertion_key(&diddoc).unwrap();

        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();
        keystore.find_keypair(&pubkey).unwrap()
    }

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
                serde_json::from_str::<DICServiceLevel>(level_str).unwrap(),
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

    #[test]
    fn can_sign_dic_payload_into_jwt() {
        let jwk = setup();

        let dic_payload: DICPayload = serde_json::from_str(
            r#"{
                "sub": "did:key:alice_identity_pub@alice_mediator",
                "iss": "did:web:alice-mediator.com:alice_mediator_pub",
                "sl": "gold",
                "nonce": "43f84868-0632-4471-b6dd-d63fa12c21f6"
            }"#,
        )
        .unwrap();

        // Sign
        let jwt = dic_payload.sign(&jwk, None).unwrap();
        let expected_jwt = concat!(
            "eyJ0eXAiOiJkaWMvdjAwMSIsImFsZyI6IkVkRFNBIn0.eyJpc3MiOiJkaWQ6d2ViOmFsaWNl",
            "LW1lZGlhdG9yLmNvbTphbGljZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZjg0ODY4LTA2",
            "MzItNDQ3MS1iNmRkLWQ2M2ZhMTJjMjFmNiIsInNsIjoiZ29sZCIsInN1YiI6ImRpZDprZXk6",
            "YWxpY2VfaWRlbnRpdHlfcHViQGFsaWNlX21lZGlhdG9yIn0.tslxNKmgVX_LhKIM5SH9KIxp",
            "_jCAXGNmjuisS2SmmGlXf2LuR3iUeAPXWm9f0XA1_jvVXw7gJLlbJFer6zSCDA"
        );
        assert_eq!(jwt, expected_jwt);

        // Verify
        let jwk = jwk.to_public();
        assert!(jws::verify_compact_jws(&jwt, &jwk).is_ok());
    }

    #[test]
    fn can_sign_ddic_payload_into_jwt() {
        let jwk = setup();

        let ddic_payload: DDICPayload = serde_json::from_str(
            r#"{
                "sub": "did:key:bob_identity_pub@alice",
                "iss": "did:web:alice-mediator.com:alice_mediator_pub",
                "dic-sub": "did:key:alice_identity_pub@alice_mediator",
                "nonce": "43f84868-0632-4471-b6dd-d63fa12c21f6"
            }"#,
        )
        .unwrap();

        // Sign
        let jwt = ddic_payload.sign(&jwk, None).unwrap();
        let expected_jwt = concat!(
            "eyJ0eXAiOiJkZGljL3YwMDEiLCJhbGciOiJFZERTQSJ9.eyJkaWMtc3ViIjoiZGlkOmtleTp",
            "hbGljZV9pZGVudGl0eV9wdWJAYWxpY2VfbWVkaWF0b3IiLCJpc3MiOiJkaWQ6d2ViOmFsaWN",
            "lLW1lZGlhdG9yLmNvbTphbGljZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZjg0ODY4LTA",
            "2MzItNDQ3MS1iNmRkLWQ2M2ZhMTJjMjFmNiIsInN1YiI6ImRpZDprZXk6Ym9iX2lkZW50aXR",
            "5X3B1YkBhbGljZSJ9.TMrKBQ22yCY-A07bIaR6c73Y9LK-rorKv9wvoh1NnYGgr2IzIvMP8g",
            "NjQmizpgjdyVXz8KlXr8F_ARl_iQ-MDA"
        );
        assert_eq!(jwt, expected_jwt);

        // Verify
        let jwk = jwk.to_public();
        assert!(jws::verify_compact_jws(&jwt, &jwk).is_ok());
    }
}
