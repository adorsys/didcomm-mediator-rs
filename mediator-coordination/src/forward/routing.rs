
use did_utils::{crypto::ed25519::Ed25519KeyPair, key_jwk::jwk::Jwk};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct Msg {
    pub message: String,
}
// construct plain message to be routed to mediator/receiver
#[derive(Serialize, Deserialize)]
pub struct Message {
    /// Payload type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub to: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_times: Option<isize>,

    pub body: Vec<String>,
    pub attachments: Msg,
}
async fn forward_message(message: Message, Sender_did: String, receiver_did: String, mediator_did: String, sender_key_pair: Ed25519KeyPair) {
    // sign 

}

#[cfg(test)]
mod test {

    use did_utils::crypto::{ed25519::Ed25519KeyPair, traits::Generate};
    use serde_json::Value;

    use crate::forward::routing::Message;

    use super::Msg;

    #[test]
    fn test_serialize_plaintext_message() {
        let msg = Msg {
            message: "Hello christian, tell me a joke".to_owned(),
        };
        assert_eq!(
            "Hello christian, tell me a joke",
            serde_json::from_value::<Value>(Value::String(msg.message)).unwrap()
        )
    }
    #[tokio::test]
    async fn test_sign_plaintext_message() {
        let msg = Msg {
            message: "Hello christian, tell me a joke".to_owned(),
        };
        let keypair = Ed25519KeyPair::new().expect("should generate keypair");
        let message = Message {
            typ: Some("https://didcomm.org/routing/2.0/forward".to_owned()),
            id: None,
            to: vec!["".to_owned()],
            expires_times: None,
            body: vec!["next".to_owned()],
            attachments: msg
        };

    }
}
