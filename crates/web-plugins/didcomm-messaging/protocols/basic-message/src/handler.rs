use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use didcomm::Message;
use shared::state::AppState;

use std::sync::Arc;

// https://didcomm.org/basicmessage/2.0/
pub fn handle_basic_message(_state: Arc<AppState>, message: Message) -> Response {
    if !message.extra_headers.contains_key("lang") {
        return (StatusCode::BAD_REQUEST, "Language is required").into_response();
    }

    StatusCode::ACCEPTED.into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use chrono::Utc;
    use did_utils::didcore::Document;
    use keystore::tests::MockKeyStore;
    use serde_json::json;
    use shared::{
        repository::tests::{MockConnectionRepository, MockMessagesRepository},
        state::AppStateRepository,
    };
    use std::{env, sync::Arc};

    #[test]
    fn test_handle_basic_message() {
        env::set_var("MASTER_KEY", "0123456789QWERTYUIOPASDFGHJKLZXC");
        let diddoc: Document = serde_json::from_str(
                r##"{
                    "@context": [
                        "https://www.w3.org/ns/did/v1",
                        "https://w3id.org/security/suites/jws-2020/v1"
                    ],
                    "id": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                    "alsoKnownAs": [
                        "did:peer:3zQmZo9aYaBjv2XtjRcTfP7X7QwyU1VVnrcEWVtcBhiAtPFa"
                    ],
                    "verificationMethod": [
                        {
                            "id": "#key-1",
                            "type": "JsonWebKey2020",
                            "controller": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                            "publicKeyJwk": {
                                "kty": "OKP",
                                "crv": "X25519",
                                "x": "_EgIPSRgbPPw5-nUsJ6xqMvw5rXn3BViGADeUrjAMzA"
                            }
                        },
                        {
                            "id": "#key-2",
                            "type": "JsonWebKey2020",
                            "controller": "did:peer:2.Ez6LSteycMr6tTki5aAEjNAVDsp1vrx9DuDWHDnky9qxyFNUF.Vz6MkigiwfSzv66VSTAeGZLsTHa8ixK1agNFvry2KjYXmg1G3.SeyJpZCI6IiNkaWRjb21tIiwicyI6eyJhIjpbImRpZGNvbW0vdjIiXSwiciI6W10sInVyaSI6Imh0dHA6Ly9hbGljZS1tZWRpYXRvci5jb20ifSwidCI6ImRtIn0",
                            "publicKeyJwk": {
                                "kty": "OKP",
                                "crv": "Ed25519",
                                "x": "PuG2L5um-tAnHlvT29gTm9Wj9fZca16vfBCPKsHB5cA"
                            }
                        }
                    ],
                    "authentication": [
                        "#key-2"
                    ],
                    "keyAgreement": [
                        "#key-1"
                    ],
                    "service": [
                        {
                            "id": "#didcomm",
                            "type": "DIDCommMessaging",
                            "serviceEndpoint": {
                                "accept": [
                                    "didcomm/v2"
                                ],
                                "routingKeys": [],
                                "uri": "http://alice-mediator.com/"
                            }
                        }
                    ]
                }"##
            ).unwrap();

        let public_domain = String::from("http://alice-mediator.com");

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
            keystore: Arc::new(MockKeyStore::new(vec![])),
        };
        let state =
            Arc::new(AppState::from(public_domain, diddoc, None, Some(repository)).unwrap());

        let message = Message::build(
            "id_alice".to_owned(),
            "https://didcomm.org/basicmessage/2.0/".to_owned(),
            json!({
               "content": "Your hovercraft is full of eels."
            }),
        )
        .header("lang".into(), json!("en"))
        .header("created_time".into(), json!(Utc::now()))
        .finalize();

        let response = handle_basic_message(state, message);

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }
}
