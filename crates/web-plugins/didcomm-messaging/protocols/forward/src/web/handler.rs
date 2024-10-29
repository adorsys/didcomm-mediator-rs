use super::routing::handler;
use axum::response::Response;
use didcomm::Message;
use shared::state::AppState;

/// Mediator receives forwarded messages, extract the next field in the message body, and the attachments in the message
/// then stores the attachment with the next field as key for pickup
pub async fn mediator_forward_process(
    state: &AppState,
    payload: Message,
) -> Result<Message, Response> {
    let result = handler(state, payload).await.unwrap();
    Ok(result)
}
