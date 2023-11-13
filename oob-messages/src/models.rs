use crate::constants::OOB_INVITATION_2_0;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use multibase::Base::Base64Url;


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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_oob() {
        let did = "test_did";

        let oob_message = OobMessage::new(did);

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
        let did = "test_did";
        let url = "test_url";
        let oob_message = OobMessage::new(did);

        let oob_url = OobMessage::serialize_oob_message(&oob_message, url);

        println!("{:?}", oob_url);

        assert!(!oob_url.is_empty());

        assert!(oob_url.starts_with(&format!("{}?_oob=", url)));
        assert!(oob_url.contains("_oob="));
    }
}
