use crate::{web::routing::handler, ForwardError};
use didcomm::Message;
use shared::state::AppState;
use std::sync::Arc;

/// Mediator receives forwarded messages, extract the next field in the message body, and the attachments in the message
/// then stores the attachment with the next field as key for pickup
pub async fn mediator_forward_process(
    state: Arc<AppState>,
    payload: Message,
) -> Result<Option<Message>, ForwardError> {
    let result = handler(state.clone(), payload).await.unwrap();
    Ok(result)
}
