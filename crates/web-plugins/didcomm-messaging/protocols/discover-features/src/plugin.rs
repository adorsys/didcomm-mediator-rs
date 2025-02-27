use crate::constants::QUERY_FEATURE;
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use didcomm::Message;
use message_api::{MessageHandler, MessagePlugin, MessageRouter};
use shared::state::AppState;
use std::sync::Arc;

/// Represents the discover-features protocol plugin
pub struct DiscoverFeaturesProtocol;

struct DiscoverFeaturesHandler;

#[async_trait]
impl MessageHandler for DiscoverFeaturesHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        crate::handler::handle_query_request(state, msg)
            .await
            .map_err(|e| e.into_response())
    }
}

impl MessagePlugin for DiscoverFeaturesProtocol {
    fn name(&self) -> &'static str {
        "discover-features"
    }

    fn didcomm_routes(&self) -> MessageRouter {
        MessageRouter::new().register(QUERY_FEATURE, DiscoverFeaturesHandler)
    }
}
