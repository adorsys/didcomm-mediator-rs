use crate::jose::jws::{self, JwsAlg, JwsError, JwsHeader};
use did_utils::key_jwk::jwk::Jwk;
use serde_json::json;

/// Wraps DIC or DDIC compact JWT into another JWT for presentation purposes
pub fn make_compact_jwt_presentation(
    content: &str,
    holder_jwk: &Jwk,
    kid: Option<&str>,
) -> Result<String, JwsError> {
    let header = jws::read_jws_header(content)?;
    let kind = match header.typ.ok_or(JwsError::UnspecifiedPayloadType)? {
        t if t.starts_with("dic/") || t == "dic" => Ok("dic"),
        t if t.starts_with("ddic/") || t == "ddic" => Ok("ddic"),
        _ => Err(JwsError::UnsupportedPayloadType),
    }?;

    let header = JwsHeader {
        alg: JwsAlg::EdDSA,
        kid: kid.map(|x| x.to_owned()),
        typ: Some(kind.to_owned() + "+jwt"),
        ..Default::default()
    };

    let payload = json!({
        kind: content,
    });

    jws::make_compact_jws(&header, payload, holder_jwk)
}

/// Verifies JWT presentation
pub fn verify_compact_jwt_presentation(content: &str, holder_jwk: &Jwk) -> Result<(), JwsError> {
    jws::verify_compact_jws(content, holder_jwk)
}

#[cfg(test)]
mod tests {
    use super::*;

    use did_endpoint::util::keystore::ToPublic;
    use multibase::Base::Base64Url;
    use serde_json::Value;

    use crate::util::{self, MockFileSystem};

    fn setup() -> Jwk {
        let mut mock_fs = MockFileSystem;

        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let (_, pubkey) = util::extract_assertion_key(&diddoc).unwrap();

        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();
        keystore.find_keypair(&pubkey).unwrap()
    }

    #[test]
    fn present_dic_via_jwt() {
        let holder_jwk = _holder_jwk();
        let dic = concat!(
            "eyJ0eXAiOiJkaWMvdjAwMSIsImFsZyI6IkVkRFNBIn0.eyJpc3MiOiJkaWQ6d2ViOmFsaWNl",
            "LW1lZGlhdG9yLmNvbTphbGljZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZjg0ODY4LTA2",
            "MzItNDQ3MS1iNmRkLWQ2M2ZhMTJjMjFmNiIsInNsIjoiZ29sZCIsInN1YiI6ImRpZDprZXk6",
            "YWxpY2VfaWRlbnRpdHlfcHViQGFsaWNlX21lZGlhdG9yIn0.tslxNKmgVX_LhKIM5SH9KIxp",
            "_jCAXGNmjuisS2SmmGlXf2LuR3iUeAPXWm9f0XA1_jvVXw7gJLlbJFer6zSCDA"
        );

        let jwt = make_compact_jwt_presentation(dic, &holder_jwk, None).unwrap();
        let expected_jwt = concat!(
            "eyJ0eXAiOiJkaWMrand0IiwiYWxnIjoiRWREU0EifQ.eyJkaWMiOiJleUowZVhBaU9pSmthV",
            "012ZGpBd01TSXNJbUZzWnlJNklrVmtSRk5CSW4wLmV5SnBjM01pT2lKa2FXUTZkMlZpT21Gc",
            "2FXTmxMVzFsWkdsaGRHOXlMbU52YlRwaGJHbGpaVjl0WldScFlYUnZjbDl3ZFdJaUxDSnViM",
            "jVqWlNJNklqUXpaamcwT0RZNExUQTJNekl0TkRRM01TMWlObVJrTFdRMk0yWmhNVEpqTWpGb",
            "U5pSXNJbk5zSWpvaVoyOXNaQ0lzSW5OMVlpSTZJbVJwWkRwclpYazZZV3hwWTJWZmFXUmxib",
            "lJwZEhsZmNIVmlRR0ZzYVdObFgyMWxaR2xoZEc5eUluMC50c2x4TkttZ1ZYX0xoS0lNNVNIO",
            "UtJeHBfakNBWEdObWp1aXNTMlNtbUdsWGYyTHVSM2lVZUFQWFdtOWYwWEExX2p2Vlh3N2dKT",
            "GxiSkZlcjZ6U0NEQSJ9.i6WiTT3K3GF3OwZrSR4E8d5MZHC3z-LW-q_okMQ4N23r7lm2MzQO",
            "aK07R220CukncjcUBTIBVIKhoS7GYNCXCA",
        );
        assert_eq!(jwt, expected_jwt);

        let header = jws::read_jws_header(&jwt).unwrap();
        assert_eq!(
            json!(header),
            json!({
                "alg": "EdDSA",
                "typ": "dic+jwt"
            })
        );

        let payload = _extract_payload(&jwt).unwrap();
        assert_eq!(
            payload,
            json!({
                "dic": dic
            })
        );
    }

    #[test]
    fn present_dic_via_jwt_with_kid() {
        let holder_jwk = _holder_jwk();
        let dic = concat!(
            "eyJ0eXAiOiJkaWMvdjAwMSIsImFsZyI6IkVkRFNBIn0.eyJpc3MiOiJkaWQ6d2ViOmFsaWNl",
            "LW1lZGlhdG9yLmNvbTphbGljZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZjg0ODY4LTA2",
            "MzItNDQ3MS1iNmRkLWQ2M2ZhMTJjMjFmNiIsInNsIjoiZ29sZCIsInN1YiI6ImRpZDprZXk6",
            "YWxpY2VfaWRlbnRpdHlfcHViQGFsaWNlX21lZGlhdG9yIn0.tslxNKmgVX_LhKIM5SH9KIxp",
            "_jCAXGNmjuisS2SmmGlXf2LuR3iUeAPXWm9f0XA1_jvVXw7gJLlbJFer6zSCDA"
        );

        let kid = "did:key:alice_identity_pub@alice_mediator";
        let jwt = make_compact_jwt_presentation(dic, &holder_jwk, Some(kid)).unwrap();

        let header = jws::read_jws_header(&jwt).unwrap();
        assert_eq!(
            json!(header),
            json!({
                "alg": "EdDSA",
                "typ": "dic+jwt",
                "kid": kid,
            })
        );
    }

    #[test]
    fn present_ddic_via_jwt() {
        let holder_jwk = _holder_jwk();
        let ddic = concat!(
            "eyJ0eXAiOiJkZGljL3YwMDEiLCJhbGciOiJFZERTQSJ9.eyJkaWMtc3ViIjoiZGlkOmtleTp",
            "hbGljZV9pZGVudGl0eV9wdWJAYWxpY2VfbWVkaWF0b3IiLCJpc3MiOiJkaWQ6d2ViOmFsaWN",
            "lLW1lZGlhdG9yLmNvbTphbGljZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZjg0ODY4LTA",
            "2MzItNDQ3MS1iNmRkLWQ2M2ZhMTJjMjFmNiIsInN1YiI6ImRpZDprZXk6Ym9iX2lkZW50aXR",
            "5X3B1YkBhbGljZSJ9.TMrKBQ22yCY-A07bIaR6c73Y9LK-rorKv9wvoh1NnYGgr2IzIvMP8g",
            "NjQmizpgjdyVXz8KlXr8F_ARl_iQ-MDA"
        );

        let jwt = make_compact_jwt_presentation(ddic, &holder_jwk, None).unwrap();
        let expected_jwt = concat!(
            "eyJ0eXAiOiJkZGljK2p3dCIsImFsZyI6IkVkRFNBIn0.eyJkZGljIjoiZXlKMGVYQWlPaUpr",
            "WkdsakwzWXdNREVpTENKaGJHY2lPaUpGWkVSVFFTSjkuZXlKa2FXTXRjM1ZpSWpvaVpHbGtP",
            "bXRsZVRwaGJHbGpaVjlwWkdWdWRHbDBlVjl3ZFdKQVlXeHBZMlZmYldWa2FXRjBiM0lpTENK",
            "cGMzTWlPaUprYVdRNmQyVmlPbUZzYVdObExXMWxaR2xoZEc5eUxtTnZiVHBoYkdsalpWOXRa",
            "V1JwWVhSdmNsOXdkV0lpTENKdWIyNWpaU0k2SWpRelpqZzBPRFk0TFRBMk16SXRORFEzTVMx",
            "aU5tUmtMV1EyTTJaaE1USmpNakZtTmlJc0luTjFZaUk2SW1ScFpEcHJaWGs2WW05aVgybGta",
            "VzUwYVhSNVgzQjFZa0JoYkdsalpTSjkuVE1yS0JRMjJ5Q1ktQTA3YklhUjZjNzNZOUxLLXJv",
            "ckt2OXd2b2gxTm5ZR2dyMkl6SXZNUDhnTmpRbWl6cGdqZHlWWHo4S2xYcjhGX0FSbF9pUS1N",
            "REEifQ.zIiqLsym6al8ypu3QBURZP_dLuU33SSuMyRZGkguqRnSSNug8ZeSIb3SEv8Iflab3",
            "YqksBincsVc36og-YSqAA",
        );
        assert_eq!(jwt, expected_jwt);

        let header = jws::read_jws_header(&jwt).unwrap();
        assert_eq!(
            json!(header),
            json!({
                "alg": "EdDSA",
                "typ": "ddic+jwt"
            })
        );

        let payload = _extract_payload(&jwt).unwrap();
        assert_eq!(
            payload,
            json!({
                "ddic": ddic
            })
        );
    }

    #[test]
    fn present_dic_with_unspecified_payload_type() {
        let holder_jwk = _holder_jwk();
        let dic = concat!(
            // typ: None
            "eyJhbGciOiJFZERTQSJ9.eyJpc3MiOiJkaWQ6d2ViOmFsaWNlLW1lZGlhdG9yLmNvbTphbGl",
            "jZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZjg0ODY4LTA2MzItNDQ3MS1iNmRkLWQ2M2Z",
            "hMTJjMjFmNiIsInNsIjoiZ29sZCIsInN1YiI6ImRpZDprZXk6YWxpY2VfaWRlbnRpdHlfcHV",
            "iQGFsaWNlX21lZGlhdG9yIn0.9wAMiyXr3VPJ0jNVc_1wFRoSAx-L86LvENBk0iy1knSHe44",
            "SDBXwNrkunrVaG288d3phiTwQxRkGFh4csdRbCA",
        );

        let kid = "did:key:alice_identity_pub@alice_mediator";
        let res = make_compact_jwt_presentation(dic, &holder_jwk, Some(kid));
        assert_eq!(res.unwrap_err(), JwsError::UnspecifiedPayloadType);
    }

    #[test]
    fn present_non_dic_or_ddic_compact_string() {
        let holder_jwk = _holder_jwk();
        let dic = concat!(
            // typ: Some("application/json")
            "eyJ0eXAiOiJhcHBsaWNhdGlvbi9qc29uIiwiYWxnIjoiRWREU0EifQ.eyJpc3MiOiJkaWQ6d",
            "2ViOmFsaWNlLW1lZGlhdG9yLmNvbTphbGljZV9tZWRpYXRvcl9wdWIiLCJub25jZSI6IjQzZ",
            "jg0ODY4LTA2MzItNDQ3MS1iNmRkLWQ2M2ZhMTJjMjFmNiIsInNsIjoiZ29sZCIsInN1YiI6I",
            "mRpZDprZXk6YWxpY2VfaWRlbnRpdHlfcHViQGFsaWNlX21lZGlhdG9yIn0.KvEHjDzT5Bgik",
            "DkZjY4_cvGesNnpPvFsqsseWpqdgkwLWii0Ao8fSl_SI3UxvyiZ8-MQ6NG2OPpyJqe8Xav8B",
            "w",
        );

        let kid = "did:key:alice_identity_pub@alice_mediator";
        let res = make_compact_jwt_presentation(dic, &holder_jwk, Some(kid));
        assert_eq!(res.unwrap_err(), JwsError::UnsupportedPayloadType);
    }

    #[test]
    fn verify_dic_presented_via_jwt() {
        let holder_jwk = _holder_jwk().to_public();
        let jwk = setup().to_public();

        let wrapping_jwt = concat!(
            "eyJhbGciOiJFZERTQSJ9.eyJkaWMiOiJleUowZVhBaU9pSmthV012ZGpBd01TSXNJbUZzWnl",
            "JNklrVmtSRk5CSW4wLmV5SnBjM01pT2lKa2FXUTZkMlZpT21Gc2FXTmxMVzFsWkdsaGRHOXl",
            "MbU52YlRwaGJHbGpaVjl0WldScFlYUnZjbDl3ZFdJaUxDSnViMjVqWlNJNklqUXpaamcwT0R",
            "ZNExUQTJNekl0TkRRM01TMWlObVJrTFdRMk0yWmhNVEpqTWpGbU5pSXNJbk5zSWpvaVoyOXN",
            "aQ0lzSW5OMVlpSTZJbVJwWkRwclpYazZZV3hwWTJWZmFXUmxiblJwZEhsZmNIVmlRR0ZzYVd",
            "ObFgyMWxaR2xoZEc5eUluMC50c2x4TkttZ1ZYX0xoS0lNNVNIOUtJeHBfakNBWEdObWp1aXN",
            "TMlNtbUdsWGYyTHVSM2lVZUFQWFdtOWYwWEExX2p2Vlh3N2dKTGxiSkZlcjZ6U0NEQSJ9.eV",
            "BZDZ7MqX_3F9n4CgvdIl2E0oWp47oLNeCwyui3lyWe9dltIl2niFctndy4nqZbs6fR3zx64A",
            "v-6lF7fDLCDg"
        );

        // Verify wrapping JWT is signed by holder
        assert!(verify_compact_jwt_presentation(wrapping_jwt, &holder_jwk).is_ok());

        // Extract DIC in payload
        let payload = _extract_payload(wrapping_jwt).unwrap();
        let dic_jwt = payload.get("dic").unwrap().as_str().unwrap();

        // Verify DIC
        assert!(jws::verify_compact_jws(dic_jwt, &jwk).is_ok());

        // Assert claims
        assert_eq!(
            _extract_payload(dic_jwt).unwrap(),
            json!({
                "sub": "did:key:alice_identity_pub@alice_mediator",
                "iss": "did:web:alice-mediator.com:alice_mediator_pub",
                "sl": "gold",
                "nonce": "43f84868-0632-4471-b6dd-d63fa12c21f6"
            })
        )
    }

    fn _holder_jwk() -> Jwk {
        serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "d": "UXBdR4u4bnHHEaDK-dqE04DIMvegx9_ZOjm--eGqHiI",
                "x": "Fpf4juyZWYUNmC8Bv87MmFLDWApxqOYYZUhWyiD7lSo"
            }"#,
        )
        .unwrap()
    }

    fn _extract_payload(jwt: &str) -> Option<Value> {
        let parts: Vec<_> = jwt.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let encoded_payload = parts[1];
        let payload = match Base64Url.decode(encoded_payload) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(content) => Some(content),
                Err(_) => None,
            },
            Err(_) => None,
        }?;

        serde_json::from_str(&payload).ok()
    }
}
