pub(crate) mod dispatcher;

use axum::{middleware, routing::post, Router};
use shared::state::AppState;
use std::sync::Arc;

use crate::midlw;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Unified route for all DIDComm messages
        .route("/", post(dispatcher::process_didcomm_message))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            midlw::unpack_didcomm_message,
        ))
        .with_state(state)
}
