use did_utils::jwk::Jwk;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum CompactDIC {
    #[serde(serialize_with = "CompactDIC::serialize_inbox_variant")]
    #[serde(deserialize_with = "CompactDIC::deserialize_inbox_variant")]
    Inbox(String),

    #[serde(serialize_with = "CompactDIC::serialize_outbox_variant")]
    #[serde(deserialize_with = "CompactDIC::deserialize_outbox_variant")]
    Outbox(String),
}

#[allow(unused)]
impl CompactDIC {
    /// Retrieve tag-less JWS string
    pub fn plain_jws(&self) -> String {
        match self {
            CompactDIC::Inbox(s) => s.clone(),
            CompactDIC::Outbox(s) => s.clone(),
        }
    }
}

macro_rules! compact_dic_variant_serder {
    ($S: ident) => {
        paste::paste! {
            fn [<serialize_ $S _variant>]<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                const S: &'static str = stringify!($S);
                serializer.serialize_str(&format!("{S}:{value}"))
            }

            fn [<deserialize_ $S _variant>]<'de, D>(deserializer: D) -> Result<String, D::Error>
            where
                D: Deserializer<'de>,
            {
                const S: &'static str = stringify!($S);
                match String::deserialize(deserializer)? {
                    s if s.starts_with(&format!("{S}:")) => Ok(s[(S.len() + 1)..].to_string()),
                    _ => Err(Error::custom("invalid tag")),
                }
            }
        }
    };
}

impl CompactDIC {
    compact_dic_variant_serder!(inbox);
    compact_dic_variant_serder!(outbox);
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
            "eyJhbGciOiJFZERTQSIsInR5cCI6ImRpYy92MDAxIn0.eyJpc3MiOiJkaWQ6d2ViOmFsaWNl",
            "LW1lZGlhdG9yLmNvbTphbGljZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZjg0ODY4LTA2",
            "MzItNDQ3MS1iNmRkLWQ2M2ZhMTJjMjFmNiIsInNsIjoiZ29sZCIsInN1YiI6ImRpZDprZXk6",
            "YWxpY2VfaWRlbnRpdHlfcHViQGFsaWNlX21lZGlhdG9yIn0.C0yYq_BE1X-B1l9Hj_jJxXXn",
            "-eLoxCzYEY6eO_2aFu2A4FausK5vZoezjoh0xaj3MEmK24XpwfrQDXrFxuXrAA",
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
            "eyJhbGciOiJFZERTQSIsInR5cCI6ImRkaWMvdjAwMSJ9.eyJkaWMtc3ViIjoiZGlkOmtleTp",
            "hbGljZV9pZGVudGl0eV9wdWJAYWxpY2VfbWVkaWF0b3IiLCJpc3MiOiJkaWQ6d2ViOmFsaWN",
            "lLW1lZGlhdG9yLmNvbTphbGljZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZjg0ODY4LTA",
            "2MzItNDQ3MS1iNmRkLWQ2M2ZhMTJjMjFmNiIsInN1YiI6ImRpZDprZXk6Ym9iX2lkZW50aXR",
            "5X3B1YkBhbGljZSJ9.gmFYNNmdnL0PX3MFVv-APnmLn55cI7GPRfkAZTcYqR76aZGEmjPFmB",
            "pDOeBZYDwM5GcDHOIB6m1xwRQ1xwvKDA",
        );
        assert_eq!(jwt, expected_jwt);

        // Verify
        let jwk = jwk.to_public();
        assert!(jws::verify_compact_jws(&jwt, &jwk).is_ok());
    }

    #[test]
    fn can_serialize_compact_dic() {
        let compact_dic = CompactDIC::Inbox(String::from("abcd123"));
        let serialized = serde_json::to_string(&compact_dic).unwrap();
        assert_eq!(serialized, r#""inbox:abcd123""#);

        let compact_dic = CompactDIC::Outbox(String::from("abcd123"));
        let serialized = serde_json::to_string(&compact_dic).unwrap();
        assert_eq!(serialized, r#""outbox:abcd123""#);
    }

    #[test]
    fn can_deserialize_compact_dic() {
        let text = r#""inbox:abcd123""#;
        let compact_dic: CompactDIC = serde_json::from_str(text).unwrap();
        assert_eq!(compact_dic, CompactDIC::Inbox(String::from("abcd123")));

        let text = r#""outbox:abcd123""#;
        let compact_dic: CompactDIC = serde_json::from_str(text).unwrap();
        assert_eq!(compact_dic, CompactDIC::Outbox(String::from("abcd123")));

        let text = r#""abcd123""#;
        let err = serde_json::from_str::<CompactDIC>(text).unwrap_err();
        assert!(err.to_string().contains("data did not match any variant"));
    }
    #[test]
    fn can_retrieve_plain_jws_from_compact_dic() {
        let compact_dic = CompactDIC::Inbox(String::from("abcd123"));
        assert_eq!(compact_dic.plain_jws(), "abcd123");

        let compact_dic = CompactDIC::Outbox(String::from("abcd123"));
        assert_eq!(compact_dic.plain_jws(), "abcd123");
    }
}
