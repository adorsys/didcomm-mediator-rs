use did_utils::{
    crypto::{ed25519::Ed25519KeyPair, traits::CoreSign},
    didcore::Jwk,
};
use multibase::Base::Base64Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(unused)]
pub enum JwsError {
    #[error("invalid format")]
    InvalidFormat,
    #[error("invalid verifying key")]
    InvalidVerifyingKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("invalid signing key")]
    InvalidSigningKey,
    #[error("missing private key")]
    MissingPrivateKey,
    #[error("signing error")]
    SigningError,
    #[error("unsupported algorithm")]
    UnsupportedAlgorithm,
    #[error("serialization error")]
    SerializationError,
    #[error("serialization error")]
    DeserializationError,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum JwsAlg {
    #[default]
    EdDSA,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct JwsHeader {
    /// Payload type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,

    /// Signature key id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,

    /// Signature algorithm
    pub alg: JwsAlg,

    /// Dynamic properties
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Issues a JSON Web Signature (JWS)
#[allow(unused)]
pub fn make_compact_jws(header: &JwsHeader, payload: Value, jwk: &Jwk) -> Result<String, JwsError> {
    let make_phrase = || -> Result<String, JwsError> {
        let encoded_header = {
            let header_json =
                serde_json::to_string(header).map_err(|_| JwsError::SerializationError)?;
            Base64Url.encode(header_json)
        };

        let encoded_payload = {
            let payload_json =
                serde_json::to_string(&payload).map_err(|_| JwsError::SerializationError)?;
            Base64Url.encode(payload_json)
        };

        Ok(format!("{encoded_header}.{encoded_payload}"))
    };

    match header.alg {
        JwsAlg::EdDSA => {
            if jwk.curve.to_ascii_lowercase() != "ed25519" {
                return Err(JwsError::InvalidSigningKey);
            }

            make_compact_jws_ed25519(make_phrase()?, jwk)
        }
        #[allow(unreachable_patterns)]
        _ => Err(JwsError::UnsupportedAlgorithm),
    }
}

pub fn make_compact_jws_ed25519(phrase: String, jwk: &Jwk) -> Result<String, JwsError> {
    let keypair: Ed25519KeyPair = jwk
        .clone()
        .try_into()
        .map_err(|_| JwsError::InvalidSigningKey)?;
    if keypair.secret_key.is_none() {
        return Err(JwsError::MissingPrivateKey);
    }

    let signature = keypair
        .sign(phrase.as_bytes())
        .map_err(|_| JwsError::SigningError)?;
    let encoded_signature = Base64Url.encode(signature);

    Ok(format!("{phrase}.{encoded_signature}"))
}

/// Verifies a JSON Web Signature (JWS)
#[allow(unused)]
pub fn verify_compact_jws(jws: &str, jwk: &Jwk) -> Result<(), JwsError> {
    let header_encoded = jws.split('.').next().ok_or(JwsError::InvalidFormat)?;
    let header_decoded = String::from_utf8(
        Base64Url
            .decode(header_encoded)
            .map_err(|_| JwsError::InvalidFormat)?,
    )
    .map_err(|_| JwsError::InvalidFormat)?;
    let header: JwsHeader =
        serde_json::from_str(&header_decoded).map_err(|_| JwsError::DeserializationError)?;

    match header.alg {
        JwsAlg::EdDSA => verify_compact_jws_ed25519(jws, jwk),
        #[allow(unreachable_patterns)]
        _ => Err(JwsError::UnsupportedAlgorithm),
    }
}

fn verify_compact_jws_ed25519(jws: &str, jwk: &Jwk) -> Result<(), JwsError> {
    let parts: Vec<_> = jws.split('.').collect();
    if parts.len() != 3 {
        return Err(JwsError::InvalidFormat);
    }

    let phrase = format!("{}.{}", parts[0], parts[1]);
    let signature_encoded = parts[2];
    let signature_decoded = Base64Url
        .decode(signature_encoded)
        .map_err(|_| JwsError::InvalidSignature)?;

    let keypair: Ed25519KeyPair = jwk
        .clone()
        .try_into()
        .map_err(|_| JwsError::InvalidSigningKey)?;

    keypair
        .verify(phrase.as_bytes(), &signature_decoded)
        .map_err(|_| JwsError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    use crate::util;

    fn setup() -> Jwk {
        let storage_dirpath = util::dotenv_flow_read("STORAGE_DIRPATH").unwrap();

        let diddoc = util::read_diddoc(&storage_dirpath).unwrap();
        let (_, pubkey) = util::extract_assertion_key(&diddoc).unwrap();

        let keystore = util::read_keystore(&storage_dirpath).unwrap();
        keystore.find_keypair(&pubkey).unwrap()
    }

    #[test]
    fn can_serde_jws_header() {
        let msg = r#"{
            "typ": "application/json",
            "kid": "did:web:mediators-r-us.com#keys-2",
            "alg": "EdDSA"
        }"#;

        // Assert deserialization
        let header: JwsHeader = serde_json::from_str(msg).unwrap();
        assert_eq!(header.typ.as_deref().unwrap(), "application/json");
        assert_eq!(
            header.kid.as_deref().unwrap(),
            "did:web:mediators-r-us.com#keys-2"
        );
        assert_eq!(header.alg, JwsAlg::EdDSA);

        // Assert re-serialization
        assert_eq!(
            json_canon::to_string(&header).unwrap(),
            json_canon::to_string(&serde_json::from_str::<Value>(msg).unwrap()).unwrap(),
        )
    }

    #[test]
    fn can_make_compact_jws() {
        let jwk = setup();

        let header = JwsHeader {
            typ: Some(String::from("application/json")),
            kid: Some(String::from("did:web:mediators-r-us.com#keys-2")),
            alg: JwsAlg::EdDSA,
            ..Default::default()
        };

        let payload = json!({
            "content": "e1200a6c-d9a2-49b4-baa6-da86d643ce3c"
        });

        let jws = make_compact_jws(&header, payload, &jwk).unwrap();
        let expected_jws = concat!(
            "eyJ0eXAiOiJhcHBsaWNhdGlvbi9qc29uIiwia2lkIjoiZGlkOndlYjptZWRpYXRvcnMtci11",
            "cy5jb20ja2V5cy0yIiwiYWxnIjoiRWREU0EifQ.eyJjb250ZW50IjoiZTEyMDBhNmMtZDlhM",
            "i00OWI0LWJhYTYtZGE4NmQ2NDNjZTNjIn0.SyWVSdFRdAu6Z-fg0hjB31MRAIQ2jBDBdU3Af",
            "Pf0Fb9Hh8CGnSWH_6yrnDDb0K1tI0YG6iSLFEHasXeCH2-iDw"
        );

        assert_eq!(jws, expected_jws);
    }

    #[test]
    fn can_make_compact_jws_v2() {
        let jwk = setup();

        let header = JwsHeader {
            alg: JwsAlg::EdDSA,
            ..Default::default()
        };

        let payload = json!({
            "content": "Edwards, what have you done?"
        });

        let jws = make_compact_jws(&header, payload, &jwk).unwrap();
        let expected_jws = concat!(
            "eyJhbGciOiJFZERTQSJ9.eyJjb250ZW50IjoiRWR3YXJkcywgd2hhdCBoYXZlIHlvdSBkb25",
            "lPyJ9.Oei0CDvmv-O0B__mMK5ZKslSG5BqOe6ITCcLcQ-wsBlGpzGqN038kofdaWbt5wZFSK",
            "lv9erZlHte_OkH7RPEAw"
        );

        assert_eq!(jws, expected_jws);
    }

    #[test]
    fn can_verify_compact_jws() {
        let jwk = Jwk { d: None, ..setup() };

        let entries = [
            concat!(
                "eyJ0eXAiOiJhcHBsaWNhdGlvbi9qc29uIiwia2lkIjoiZGlkOndlYjptZWRpYXRvcnMtci11",
                "cy5jb20ja2V5cy0yIiwiYWxnIjoiRWREU0EifQ.eyJjb250ZW50IjoiZTEyMDBhNmMtZDlhM",
                "i00OWI0LWJhYTYtZGE4NmQ2NDNjZTNjIn0.SyWVSdFRdAu6Z-fg0hjB31MRAIQ2jBDBdU3Af",
                "Pf0Fb9Hh8CGnSWH_6yrnDDb0K1tI0YG6iSLFEHasXeCH2-iDw"
            ),
            concat!(
                "eyJhbGciOiJFZERTQSJ9.eyJjb250ZW50IjoiRWR3YXJkcywgd2hhdCBoYXZlIHlvdSBkb25",
                "lPyJ9.Oei0CDvmv-O0B__mMK5ZKslSG5BqOe6ITCcLcQ-wsBlGpzGqN038kofdaWbt5wZFSK",
                "lv9erZlHte_OkH7RPEAw"
            ),
        ];

        for entry in entries {
            assert!(verify_compact_jws(entry, &jwk).is_ok());
        }
    }
}
