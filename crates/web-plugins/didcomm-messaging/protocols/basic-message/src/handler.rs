use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use shared::state::AppState;

use std::sync::Arc;

use crate::model::BasicMessage;

pub fn handle_basic_message(
    _state: Arc<AppState>,
    _message: BasicMessage,
) -> Response {
    
    StatusCode::ACCEPTED.into_response()
}

    #[cfg(test)]
    mod tests {
        use super::*;
        use axum::http::StatusCode;
        use did_utils::didcore::Document;
        use keystore::tests::MockKeyStore;
        use shared::{repository::tests::{MockConnectionRepository, MockMessagesRepository}, state::AppStateRepository};
        use std::sync::Arc;
        use chrono::{DateTime, Utc};
        use serde_json::Value;
    
    
        #[test]
        fn test_handle_basic_message() {

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
            let state = Arc::new(AppState::from(public_domain, diddoc, Some(repository)));
    
            let created_time = "2024-11-22T10:00:00Z"
                .parse::<DateTime<Utc>>()
                .expect("Failed to parse datetime");
            
            let message = BasicMessage {
                id: "1".to_string(),
                message_type: "text".to_string(),
                lang: Some("en".to_string()),
                created_time,
                body: Value::String("Test message body".to_string()),
            };
    
            let response = handle_basic_message(state, message);
    
            assert_eq!(response.status(), StatusCode::ACCEPTED);
        }
}
