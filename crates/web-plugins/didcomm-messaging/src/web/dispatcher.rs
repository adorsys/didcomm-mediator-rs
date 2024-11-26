use crate::{constants::DIDCOMM_ENCRYPTED_MIME_TYPE, plugin::MESSAGE_CONTAINER};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};
use didcomm::Message;
use hyper::{header::CONTENT_TYPE, StatusCode};
use shared::state::AppState;
use std::sync::Arc;

#[axum::debug_handler]
pub(crate) async fn process_didcomm_message(
    State(state): State<Arc<AppState>>,
    Extension(message): Extension<Message>,
) -> Response {
    if let Some(handler) = MESSAGE_CONTAINER
        .get()
        .unwrap()
        .read()
        .await
        .didcomm_routes()
        .unwrap_or_default()
        .get_handler(&message.type_)
    {
        let response = handler.handle(state.clone(), message).await;
        return process_response(state, response).await;
    }

    (StatusCode::BAD_REQUEST, "Unsupported didcomm message").into_response()
}

async fn process_response(
    state: Arc<AppState>,
    response: Result<Option<Message>, Response>,
) -> Response {
    match response {
        Ok(message) => match message {
            Some(message) => crate::midlw::pack_response_message(
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
            None => StatusCode::ACCEPTED.into_response(),
        },
        Err(response) => response,
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::manager::MessagePluginContainer;
    use axum::Router;
    use hyper::{Body, Method, Request};
    use mediator_coordination::handler;
    use message_api::{MessageHandler, MessagePlugin, MessageRouter};
    use once_cell::sync::Lazy;
    use serde_json::{json, Value};
    use shared::{
        repository::tests::MockConnectionRepository, state::AppStateRepository,
        utils::tests_utils::tests as global,
    };
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    #[allow(clippy::needless_update)]
    pub fn setup() -> (Router, Arc<AppState>) {
        let state = global::setup();

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
        let app = crate::web::routes(state.clone());

        (app, state)
    }

    #[derive(Debug)]
    struct MockKeylistUpdateHandler;
    struct MockProtocol;

    #[async_trait::async_trait]
    impl MessageHandler for MockKeylistUpdateHandler {
        async fn handle(
            &self,
            state: Arc<AppState>,
            message: Message,
        ) -> Result<Option<Message>, Response> {
            handler::stateful::process_plain_keylist_update_message(state, message)
                .await
                .map_err(|e| e.into_response())
        }
    }

    impl MessagePlugin for MockProtocol {
        fn name(&self) -> &'static str {
            "mock_protocol"
        }

        fn didcomm_routes(&self) -> MessageRouter {
            MessageRouter::new().register(
                "https://didcomm.org/coordinate-mediation/2.0/keylist-update",
                MockKeylistUpdateHandler,
            )
        }
    }

    static MOCK_PLUGINS: Lazy<Vec<Arc<dyn MessagePlugin>>> =
        Lazy::new(|| vec![Arc::new(MockProtocol)]);

    #[tokio::test]
    async fn test_keylist_update_via_didcomm() {
        let mut container = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &MOCK_PLUGINS,
        };

        assert!(container.load().is_ok());

        if let Err(_) = MESSAGE_CONTAINER.set(RwLock::new(container)) {
            panic!("Failed to initialize MESSAGE_CONTAINER");
        }

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
                    .uri(String::from("/"))
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
        assert_eq!(
            response.type_,
            "https://didcomm.org/coordinate-mediation/2.0/keylist-update-response"
        );
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
}
