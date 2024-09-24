use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};
use didcomm::Message;
use hyper::{header::CONTENT_TYPE, StatusCode};
use std::sync::Arc;

use crate::{
    constant::{
        DIDCOMM_ENCRYPTED_MIME_TYPE, KEYLIST_QUERY_2_0, KEYLIST_UPDATE_2_0, MEDIATE_FORWARD_2_0,
        MEDIATE_REQUEST_2_0,
    },
    forward::routing::mediator_forward_process, web::{self, error::MediationError, AppState},
    
   
};

#[axum::debug_handler]
pub(crate) async fn process_didcomm_message(
    State(state): State<Arc<AppState>>,
    Extension(message): Extension<Message>,
) -> Response {
    if message.type_ == MEDIATE_FORWARD_2_0 {
        let response = mediator_forward_process(&state, message)
            .await
            .map(|_| StatusCode::ACCEPTED.into_response())
            .map_err(|err| err);

        return match response {
            Ok(_message) => StatusCode::ACCEPTED.into_response(),
            Err(response) => response,
        };
    }
    let response = match message.type_.as_str() {
        KEYLIST_UPDATE_2_0 => {
            web::coord::handler::stateful::process_plain_keylist_update_message(
                Arc::clone(&state),
                message,
            )
            .await
        },
        KEYLIST_QUERY_2_0 => {
            web::coord::handler::stateful::process_plain_keylist_query_message(
                Arc::clone(&state),
                message,
            )
            .await
        }
        KEYLIST_QUERY_2_0 => {
            web::coord::handler::stateful::process_plain_keylist_update_message(
                Arc::clone(&state),
                message,
            )
            .await
        }
        MEDIATE_REQUEST_2_0 => {
            web::coord::handler::stateful::process_mediate_request(&state, &message).await
        }
 
       
        _ => {
            let response = (
                StatusCode::BAD_REQUEST,
                MediationError::UnsupportedOperation.json(),
            );
            return response.into_response();
        }
    };

    process_response(state, response).await
}

async fn process_response(state: Arc<AppState>, response: Result<Message, Response>) -> Response {
    match response {
        Ok(message) => web::midlw::pack_response_message(
            &message,
            &state.did_resolver,
            &state.secrets_resolver,
        )
        .await
        .map(|packed| {
            (
                StatusCode::ACCEPTED,
                [(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)],
                Json(packed),
            )
                .into_response()
        })
        .unwrap_or_else(|err| err.into_response()),
        Err(response) => response,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use axum::Router;
    use did_utils::jwk::Jwk;
    use didcomm::{
        error::Error as DidcommError, secrets::SecretsResolver, Message, PackEncryptedOptions,
        UnpackOptions,
    };
    use web::AppStateRepository;

    use crate::{
        didcomm::bridge::LocalSecretsResolver, repository::stateful::coord::tests::{MockConnectionRepository, MockMessagesRepository, MockSecretsRepository}, util::{self, MockFileSystem}
      
    };

    pub fn setup() -> (Router, Arc<AppState>) {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            secret_repository: Arc::new(MockSecretsRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
        };

        let state = Arc::new(AppState::from(
            public_domain,
            diddoc,
            keystore,
            Some(repository),
        ));
        let app = web::routes(Arc::clone(&state));

        (app, state)
    }

    pub fn _mediator_did(state: &AppState) -> String {
        state.diddoc.id.clone()
    }

    pub fn _edge_did() -> String {
        "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
    }

    pub fn _edge_signing_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "d": "UXBdR4u4bnHHEaDK-dqE04DIMvegx9_ZOjm--eGqHiI",
                "x": "Fpf4juyZWYUNmC8Bv87MmFLDWApxqOYYZUhWyiD7lSo"
            }"#,
        )
        .unwrap();

        LocalSecretsResolver::new(secret_id, &secret)
    }

    pub fn _edge_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU",
                "d": "oItI6Jx-anGyhiDJIXtVAhzugOha05s-7_a5_CTs_V4"
            }"#,
        )
        .unwrap();

        LocalSecretsResolver::new(secret_id, &secret)
    }

    pub async fn _edge_pack_message(
        state: &AppState,
        msg: &Message,
        from: Option<String>,
        to: String,
    ) -> Result<String, DidcommError> {
        let (packed, _) = msg
            .pack_encrypted(
                &to,
                from.as_deref(),
                None,
                &state.did_resolver,
                &_edge_secrets_resolver(),
                &PackEncryptedOptions::default(),
            )
            .await?;

        Ok(packed)
    }

    pub async fn _edge_unpack_message(
        state: &AppState,
        msg: &str,
    ) -> Result<Message, DidcommError> {
        let (unpacked, _) = Message::unpack(
            msg,
            &state.did_resolver,
            &_edge_secrets_resolver(),
            &UnpackOptions::default(),
        )
        .await
        .expect("Unable to unpack");

        Ok(unpacked)
    }
}

#[cfg(test)]
mod tests2 {
    use super::{tests as global, *};
    use crate::{
        constant::{KEYLIST_UPDATE_RESPONSE_2_0, MEDIATE_GRANT_2_0}, repository::stateful::coord::tests::MockConnectionRepository, web::{self, AppStateRepository}
    };

    use axum::{
        body::Body,
        http::{Method, Request},
        Router,
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    #[allow(clippy::needless_update)]
    pub fn setup() -> (Router, Arc<AppState>) {
        let (_, state) = global::setup();

        let mut state = match Arc::try_unwrap(state) {
            Ok(state) => state,
            Err(_) => panic!(),
        };

        state.repository = Some(AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(
                serde_json::from_str(
                    r##"[
                      {
                        "_id": {
                            "$oid": "6580701fd2d92bb3cd291b2a"
                        },
                        "client_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
                        "mediator_did": "did:web:alice-mediator.com:alice_mediator_pub",
                        "routing_did": "did:key:generated",
                        "keylist": [
                            "did:key:alice_identity_pub1@alice_mediator"
                        ]
                    }
                ]"##,
                )
                .unwrap(),
            )),
            ..state.repository.unwrap()
        });

        let state = Arc::new(state);
        let app = web::routes(Arc::clone(&state));

        (app, state)
    }

    #[tokio::test]
    async fn test_keylist_update_via_didcomm() {
        let (app, state) = setup();

        // Build message
        let msg = Message::build(
            "id_alice_keylist_update_request".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
            json!({
                "updates": [
                    {
                        "action": "remove",
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator"
                    },
                    {
                        "action": "add",
                        "recipient_did": "did:key:alice_identity_pub2@alice_mediator"
                    },
                ]
            }),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Encrypt message for mediator
        let packed_msg = global::_edge_pack_message(
            &state,
            &msg,
            Some(global::_edge_did()),
            global::_mediator_did(&state),
        )
        .await
        .unwrap();

        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert response's metadata
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            DIDCOMM_ENCRYPTED_MIME_TYPE
        );

        // Parse response's body
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        let response = serde_json::to_string(&body).unwrap();

        // Decrypt response
        let response: Message = global::_edge_unpack_message(&state, &response)
            .await
            .unwrap();

        // Assert metadata
        assert_eq!(response.type_, KEYLIST_UPDATE_RESPONSE_2_0);
        assert_eq!(response.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(response.to.unwrap(), vec![global::_edge_did()]);

        // Assert updates
        assert_eq!(
            response.body,
            json!({
                "updated": [
                    {
                        "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                        "action": "remove",
                        "result": "success"
                    },
                    {
                        "recipient_did":"did:key:alice_identity_pub2@alice_mediator",
                        "action": "add",
                        "result": "success"
                    },
                ]
            })
        );
    }
    #[tokio::test]
    async fn test_mediate_request() {
        let (app, state) = setup();

        // Build message
        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/mediate-request".to_owned(),
            json!({}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        // Encrypt message for mediator

        let packed_msg = global::_edge_pack_message(
            &state,
            &msg,
            Some(global::_edge_did()),
            global::_mediator_did(&state),
        )
        .await
        .unwrap();

        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert response's metadata
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            DIDCOMM_ENCRYPTED_MIME_TYPE
        );

        // Parse response's body
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        let response = serde_json::to_string(&body).unwrap();

        // Decrypt response
        let response: Message = global::_edge_unpack_message(&state, &response)
            .await
            .unwrap();

        // Assert metadata
        assert_eq!(response.type_, MEDIATE_GRANT_2_0);
        assert_eq!(response.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(response.to.unwrap(), vec![global::_edge_did()]);

        // Assert updates
        // assert_eq!(
        //     response.body,
        //     json!({
        //         "updated": [
        //             {
        //                 "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
        //                 "action": "remove",
        //                 "result": "success"
        //             },
        //             {
        //                 "recipient_did":"did:key:alice_identity_pub2@alice_mediator",
        //                 "action": "add",
        //                 "result": "success"
        //             },
        //         ]
        //     })
        // );
        
    }
    #[tokio::test]
    async fn test_keylist_query_success() {
        let state = setup();

        // Prepare request
        let message = Message::build(
            "id_alice_keylist_query".to_owned(),
            "https://didcomm.org/coordinate-mediation/2.0/keylist-query".to_owned(),
            json!({}),
        )
        .to(global::_mediator_did(&state.1))
        .from(global::_edge_did())
        .finalize();

        // Encrypt message for mediator
        let packed_msg = global::_edge_pack_message(
            &state.1,
            &message,
            Some(global::_edge_did()),
            global::_mediator_did(&state.1),
        )
        .await
        .unwrap();

        println!("{}", packed_msg);
    }

    #[tokio::test]
    async fn test_pickup_test() {
        let (_app, state) = setup();
        // Build message
        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            "https://didcomm.org/messagepickup/3.0/status-request".to_owned(),
            json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
        )
        .header("return_route".into(), json!("all"))
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let packed_msg = global::_edge_pack_message(
            &state,
            &msg,
            Some(global::_edge_did()),
            global::_mediator_did(&state),
        )
        .await
        .unwrap();
        println!("{}", packed_msg);
    }
}
