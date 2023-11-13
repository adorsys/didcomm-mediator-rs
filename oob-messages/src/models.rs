use crate::constants::OOB_INVITATION_2_0;
use crate::util::dotenv_flow_read;
use multibase::Base::Base64Url;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::error::Error;
use url::{ParseError, Url};

// region: --- Model

// OOB Inv: Is an unencrypted message (In the form of an URL or QR code that contains a b64urlencode JWM) with the mediator public DID.

// The out-of-band protocol consists in a single message that is sent by the sender.

// This is the first step in the interaction with the Mediator. The following one is the mediation coordination where a 'request mediation' request is created and performed.
/// e.g.:
/// ```
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
/// ```
///
///
///

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

    fn serialize_oob_message(oob_message: &OobMessage, url: &str) -> String {
        let plaintext = to_string(oob_message).unwrap();
        let encoded_jwm = Base64Url.encode(plaintext.as_bytes());

        format!("{}?_oob={}", url, encoded_jwm)
    }
}

pub fn generate_from_field() -> String {
    let server_public_domain = dotenv_flow_read("SERVER_PUBLIC_DOMAIN").unwrap();
    let server_local_port = dotenv_flow_read("SERVER_LOCAL_PORT").unwrap();

    url_to_did_web_id(&format!(
        "{}:{}/.well-known/did/pop.json",
        server_public_domain, server_local_port
    ))
    .unwrap()
}

/// Turns an HTTP(S) URL into a did:web id.
pub fn url_to_did_web_id(url: &str) -> Result<String, Box<dyn Error>> {
    let url = url.trim();

    let parsed = if url.contains("://") {
        if ["http://", "https://"].iter().all(|x| !url.starts_with(x)) {
            return Err("Scheme not allowed")?;
        }
        Url::parse(url)?
    } else {
        Url::parse(&format!("http://{url}"))?
    };

    let domain = parsed.domain().ok_or(ParseError::EmptyHost)?;

    let mut port = String::new();
    if let Some(parsed_port) = parsed.port() {
        port = format!("%3A{parsed_port}");
    }

    let mut path = parsed.path().replace('/', ":");
    if path.len() == 1 {
        // Discards single '/' character
        path = String::new();
    }

    Ok(format!("did:web:{domain}{port}{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_oob() {
        let did = generate_from_field();

        let oob_message = OobMessage::new(&did);

        let json_string = serde_json::to_string(&oob_message).unwrap();
        println!("{}", json_string);

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
        let did = generate_from_field();
        let url = "test_url";
        let oob_message = OobMessage::new(&did);

        let oob_url = OobMessage::serialize_oob_message(&oob_message, url);

        println!("{:?}", oob_url);

        assert!(!oob_url.is_empty());

        assert!(oob_url.starts_with(&format!("{}?_oob=", url)));
        assert!(oob_url.contains("_oob="));
    }
}
