use crate::constants::TRUST_PING_2_0;
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use didcomm::Message;
use message_api::{MessageHandler, MessagePlugin, MessageRouter};
use shared::state::AppState;
use std::sync::Arc;

/// Represents the trust-ping protocol plugin
pub struct TrustPingProtocol;

struct TrustPingHandler;

#[async_trait]
impl MessageHandler for TrustPingHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        crate::handler::handle_trust_ping(state, msg)
            .await
            .map_err(|e| e.into_response())
    }
}

impl MessagePlugin for TrustPingProtocol {
    fn name(&self) -> &'static str {
        "trust-ping"
    }

    fn didcomm_routes(&self) -> MessageRouter {
        MessageRouter::new().register(TRUST_PING_2_0, TrustPingHandler)
    }
}
