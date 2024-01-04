use std::sync::Arc;

use axum::{
    Router,
    routing::get,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    middleware::{self, Next},
    extract::{State},
};

use super::AppState;

pub async fn unpack_didcomm_message<B>(
    State(state): State<Arc<AppState>>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // do something with `request`...

    let response = next.run(request).await;

    // do something with `response`...

    response
}
