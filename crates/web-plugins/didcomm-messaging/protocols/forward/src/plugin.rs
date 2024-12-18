use crate::constants::MEDIATE_FORWARD_2_0;
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use didcomm::Message;
use message_api::{MessageHandler, MessagePlugin, MessageRouter};
use shared::{circuit_breaker::CircuitBreaker, state::AppState};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub struct RoutingProtocol;

struct ForwardHandler;

#[async_trait]
impl MessageHandler for ForwardHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        let circuit_breaker = Arc::new(Mutex::new(CircuitBreaker::new(
            2,
            Duration::from_millis(5000),
        )));

        // Pass the state, msg, and the circuit_breaker as arguments
        crate::handler::mediator_forward_process(state, msg, circuit_breaker)
            .await
            .map_err(|e| e.into_response())
    }
}

impl MessagePlugin for RoutingProtocol {
    fn name(&self) -> &'static str {
        "routing"
    }

    fn didcomm_routes(&self) -> MessageRouter {
        MessageRouter::new().register(MEDIATE_FORWARD_2_0, ForwardHandler)
    }
}
