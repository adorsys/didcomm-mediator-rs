use crate::constants::OOB_INVITATION_2_0;
use crate::store::Store;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use did_utils::didcore::Document;
use image::{ImageFormat, Luma};
use multibase::Base::Base64Url;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::io::Cursor;

// region: --- Model

// OOB Inv: Is an unencrypted message (In the form of an URL or QR code that contains a b64urlencode JWM) with the mediator public DID.

// The out-of-band protocol consists in a single message that is sent by the sender.

// This is the first step in the interaction with the Mediator. The following one is the mediation coordination where a 'request mediation' request is created and performed.
// e.g.:
// ```
// {
//     "type": "https://didcomm.org/out-of-band/2.0/invitation",
//     "id": "0a2c57a5-5662-48a8-bca8-78275cef3c80",
//     "from": "did:peer:2.Ez6LSms555YhFthn1WV8ciDBpZm86hK9tp83WojJUmxPGk1hZ.Vz6MkmdBjMyB4TS5UbbQw54szm8yvMMf1ftGV2sQVYAxaeWhE.SeyJpZCI6Im5ldy1pZCIsInQiOiJkbSIsInMiOiJodHRwczovL21lZGlhdG9yLnJvb3RzaWQuY2xvdWQiLCJhIjpbImRpZGNvbW0vdjIiXX0",
//     "body": {
//       "goal_code": "request-mediate",
//       "goal": "RequestMediate",
//       "label": "Mediator",
//       "accept": [
//         "didcomm/v2"
//       ]
//     }
//   }
// ```

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OobMessage {
    #[serde(rename = "type")]
    oob_type: String,
    id: String,
    from: String,
    body: Body,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "use")]
struct Body {
    goal_code: String,
    goal: String,
    label: String,
    accept: Vec<String>,
}

impl OobMessage {
    fn new(did: &str) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let body = Body {
            goal_code: String::from("request-mediate"),
            goal: String::from("Request Mediate"),
            label: String::from("Mediator"),
            accept: vec![String::from("didcomm/v2")],
        };

        OobMessage {
            oob_type: String::from(OOB_INVITATION_2_0),
            id,
            from: String::from(did),
            body,
        }
    }

    fn serialize_oob_message(oob_message: &OobMessage, url: &str) -> Result<String, String> {
        let plaintext =
            to_string(oob_message).map_err(|e| format!("Serialization error: {}", e))?;
        let encoded_jwm = Base64Url.encode(plaintext.as_bytes());

        Ok(format!("{}?_oob={}", url, encoded_jwm))
    }
}

// Receives server path/port and returns a String with the OOB URL.
pub(crate) fn retrieve_or_generate_oob_inv<S>(
    store: &mut S,
    diddoc: &Document,
    server_public_domain: &str,
) -> Result<String, String>
where
    S: Store + ?Sized,
{
    if let Some(content) = store.get("oob_invitation") {
        tracing::info!("OOB Invitation successfully retrieved from store");
        return Ok(content);
    }

    let did = diddoc.id.clone();
    let oob_message = OobMessage::new(&did);
    let oob_url = OobMessage::serialize_oob_message(&oob_message, server_public_domain)
        .map_err(|err| format!("Serialization error: {err}"))?;

    store.set("oob_invitation", &oob_url);

    Ok(oob_url)
}

// Function to generate and save a QR code image with caching
pub(crate) fn retrieve_or_generate_qr_image<S>(store: &mut S, url: &str) -> Result<String, String>
where
    S: Store + ?Sized,
{
    if let Some(image_data) = store.get("qr_code_image") {
        tracing::info!("QR code image successfully retrieved from store");
        return Ok(image_data);
    }

    let code =
        QrCode::new(url.as_bytes()).map_err(|err| format!("QR code generation error: {err}"))?;
    let image = code.render::<Luma<u8>>().build();

    let mut buffer = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|err| format!("Image write error: {err}"))?;

    let image_data = STANDARD.encode(&buffer);
    store.set("qr_code_image", &image_data);

    Ok(image_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemoryStore;
    use did_utils::didcore::Document;

    #[test]
    fn test_create_oob_message() {
        let did = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr";

        let oob_message = OobMessage::new(did);

        assert_eq!(oob_message.oob_type, OOB_INVITATION_2_0);
        assert!(!oob_message.id.is_empty());
        assert_eq!(oob_message.from, did);
        assert_eq!(oob_message.body.goal_code, "request-mediate");
        assert_eq!(oob_message.body.goal, "Request Mediate");
        assert_eq!(oob_message.body.label, "Mediator");
        assert_eq!(oob_message.body.accept, vec!["didcomm/v2"]);
    }

    #[test]
    fn test_serialize_oob_message() {
        let did = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr";

        let server_public_domain = "https://example.com";
        let server_local_port = "8080";

        let url = format!("{}:{}", server_public_domain, server_local_port);

        let oob_message = OobMessage::new(did);

        let oob_url = OobMessage::serialize_oob_message(&oob_message, &url)
            .unwrap_or_else(|err| panic!("Failed to serialize oob message: {}", err));

        assert!(!oob_url.is_empty());
        assert!(oob_url.starts_with(&format!("{}?_oob=", url)));
        assert!(oob_url.contains("_oob="));
    }

    #[test]
    fn test_retrieve_or_generate_oob_inv() {
        // Test data
        let server_public_domain = "https://example.com";
        let diddoc: Document = serde_json::from_str(
            r##"{
            "@context": ["https://www.w3.org/ns/did/v1"],
            "id": "did:peer:123"
        }"##,
        )
        .unwrap();

        let mut store = InMemoryStore::default();

        let result = retrieve_or_generate_oob_inv(&mut store, &diddoc, server_public_domain);

        assert!(result.is_ok());
        let oob_inv = result.unwrap();
        assert!(oob_inv.starts_with("https://example.com?_oob="));

        // test retrieval
        let result2 = retrieve_or_generate_oob_inv(&mut store, &diddoc, server_public_domain);
        assert!(result2.is_ok());
        assert_eq!(oob_inv, result2.unwrap());
    }

    #[test]
    fn test_retrieve_or_generate_qr_image() {
        let mut store = InMemoryStore::default();
        let url = "https://example.com";

        let result = retrieve_or_generate_qr_image(&mut store, url);
        assert!(result.is_ok());
        let image_data = result.unwrap();
        assert!(!image_data.is_empty());

        // test retrieval
        let result2 = retrieve_or_generate_qr_image(&mut store, url);
        assert!(result2.is_ok());
        assert_eq!(image_data, result2.unwrap());
    }
}
