use super::midlw::{self};
use crate::{
    constant::{MEDIATE_DENY_2_0, MEDIATE_GRANT_2_0, MEDIATE_REQUEST_2_0},
    model::stateful::coord::{MediationDeny, MediationRequest},
    web::AppState,
};
use axum::response::Response;
use didcomm::Message;
use serde_json::json;
use uuid::Uuid;

/// Process a DIC-wise mediation request
pub async fn process_mediate_request(
    state: &AppState,
    plain_message: &Message,
    mediation_request: &MediationRequest,
) -> Result<Message, Response> {
    let mediator_did = &state.diddoc.id;

    let sender_did = plain_message
        .from
        .as_ref()
        .expect("should not panic as anonymous requests are rejected earlier");

    // If there is already mediation, send denial
    return Ok(Message::build(
        format!("urn:uuid:{}", Uuid::new_v4()),
        MEDIATE_DENY_2_0.to_string(),
        json!(MediationDeny {
            id: format!("urn:uuid:{}", Uuid::new_v4()),
            message_type: MEDIATE_DENY_2_0.to_string(),
            ..Default::default()
        }),
    )
    .to(sender_did.clone())
    .from(mediator_did.clone())
    .finalize());

    // Create routing, store it

    // Send mediation grant
}
