use axum::response::Response;
use didcomm::Message;
use mediator_coordination::web::AppState;
use serde_json::Value;
use crate::model::{StatusRequest, ReturnRoute};

// Process pickup status request
pub(crate) fn handle_status_request(
    state: &AppState,
    message: &Message,
) -> Result<Message, Response> {
    let recipient_did = message.body.get("recipient_did").and_then(Value::as_str);
}
