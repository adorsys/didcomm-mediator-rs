mod handler;

use axum::{middleware, routing::post, Router};
use mediator_coordination::web::{unpack_didcomm_message, AppState};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Unified route for all DIDComm messages
        .route("/", post(handler::handle_message_pickup))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            unpack_didcomm_message,
        ))
        .with_state(state)
}
