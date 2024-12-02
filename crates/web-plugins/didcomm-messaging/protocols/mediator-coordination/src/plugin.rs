use crate::constants::{KEYLIST_QUERY_2_0, KEYLIST_UPDATE_2_0, MEDIATE_REQUEST_2_0};
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use didcomm::Message;
use message_api::{MessageHandler, MessagePlugin, MessageRouter};
use shared::state::AppState;
use std::sync::Arc;

pub struct MediatorCoordinationProtocol;

struct MediateRequestHandler;
struct KeylistUpdateHandler;
struct KeylistQueryHandler;

#[async_trait]
impl MessageHandler for MediateRequestHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        crate::handler::stateful::process_mediate_request(state, &msg)
            .await
            .map_err(|e| e.into_response())
    }
}

#[async_trait]
impl MessageHandler for KeylistUpdateHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        crate::handler::stateful::process_plain_keylist_update_message(state, msg)
            .await
            .map_err(|e| e.into_response())
    }
}

#[async_trait]
impl MessageHandler for KeylistQueryHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        crate::handler::stateful::process_plain_keylist_query_message(state, msg)
            .await
            .map_err(|e| e.into_response())
    }
}

impl MessagePlugin for MediatorCoordinationProtocol {
    fn name(&self) -> &'static str {
        "mediator-coordination"
    }

    fn didcomm_routes(&self) -> MessageRouter {
        MessageRouter::new()
            .register(MEDIATE_REQUEST_2_0, MediateRequestHandler)
            .register(KEYLIST_UPDATE_2_0, KeylistUpdateHandler)
            .register(KEYLIST_QUERY_2_0, KeylistQueryHandler)
    }
}
