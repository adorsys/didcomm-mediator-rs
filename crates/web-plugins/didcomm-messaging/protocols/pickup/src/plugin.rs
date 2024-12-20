use crate::constants::{
    DELIVERY_REQUEST_3_0, LIVE_MODE_CHANGE_3_0, MESSAGE_RECEIVED_3_0, STATUS_REQUEST_3_0,
};
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use didcomm::Message;
use message_api::{MessageHandler, MessagePlugin, MessageRouter};
use shared::{circuit_breaker::CircuitBreaker, state::AppState};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

/// Represents the pickup protocol plugin.
pub struct PickupProtocol;

struct StatusRequestHandler;
struct DeliveryRequestHandler;
struct MessageReceivedHandler;
struct LiveModeChangeHandler;

#[async_trait]
impl MessageHandler for StatusRequestHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        let circuit_breaker = Arc::new(Mutex::new(CircuitBreaker::new(
            2,
            Duration::from_millis(5000),
        )));

        crate::handler::handle_status_request(state, msg, circuit_breaker)
            .await
            .map_err(|e| e.into_response())
    }
}

#[async_trait]
impl MessageHandler for DeliveryRequestHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        let circuit_breaker = Arc::new(Mutex::new(CircuitBreaker::new(
            2,
            Duration::from_millis(5000),
        )));

        crate::handler::handle_delivery_request(state, msg, circuit_breaker)
            .await
            .map_err(|e| e.into_response())
    }
}

#[async_trait]
impl MessageHandler for MessageReceivedHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        let circuit_breaker = Arc::new(Mutex::new(CircuitBreaker::new(
            2,
            Duration::from_millis(5000),
        )));

        crate::handler::handle_message_acknowledgement(state, msg, circuit_breaker)
            .await
            .map_err(|e| e.into_response())
    }
}

#[async_trait]
impl MessageHandler for LiveModeChangeHandler {
    async fn handle(
        &self,
        state: Arc<AppState>,
        msg: Message,
    ) -> Result<Option<Message>, Response> {
        crate::handler::handle_live_delivery_change(state, msg)
            .await
            .map_err(|e| e.into_response())
    }
}

impl MessagePlugin for PickupProtocol {
    fn name(&self) -> &'static str {
        "pickup"
    }

    fn didcomm_routes(&self) -> MessageRouter {
        MessageRouter::new()
            .register(STATUS_REQUEST_3_0, StatusRequestHandler)
            .register(DELIVERY_REQUEST_3_0, DeliveryRequestHandler)
            .register(MESSAGE_RECEIVED_3_0, MessageReceivedHandler)
            .register(LIVE_MODE_CHANGE_3_0, LiveModeChangeHandler)
    }
}
