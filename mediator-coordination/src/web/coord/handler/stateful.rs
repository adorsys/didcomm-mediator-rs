use crate::{
    constant::{MEDIATE_DENY_2_0, MEDIATE_GRANT_2_0},
    model::stateful::coord::{MediationDeny, MediationRequest, MediationGrant},
    web::AppState,
};
use axum::response::Response;
use didcomm::Message;
use serde_json::json;
use uuid::Uuid;
use did_utils::methods::did_key::DIDKeyMethod;

const IS_THERE_EXISTING_CONNECTION: bool = false;

/// Process a DIC-wise mediation request
pub async fn process_mediate_request(
    state: &AppState,
    plain_message: &Message,
    mediation_request: &MediationRequest
) -> Result<Message, Response> {
    let mediator_did = &state.diddoc.id;
    println!("mediation_request: {:#?}", mediation_request);

    let sender_did = plain_message
        .from
        .as_ref()
        .expect("should not panic as anonymous requests are rejected earlier");

    // This will be replaced by a proper DB check    
    // If there is already mediation, send denial
    if IS_THERE_EXISTING_CONNECTION {
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
    } else {
        /* Issue mediate grant response */

        // Create routing, store it and send mediation grant
        let did = DIDKeyMethod::generate();

        let mediation_grant = MediationGrant {
            id: format!("urn:uuid:{}", Uuid::new_v4()),
            message_type: MEDIATE_GRANT_2_0.to_string(),
            routing_did: did.unwrap().to_string(),
            ..Default::default()
        };

        //TO-DO: Store it
    
        Ok(Message::build(
            format!("urn:uuid:{}", Uuid::new_v4()),
            mediation_grant.message_type.clone(),
            json!(mediation_grant),
        )
        .to(sender_did.clone())
        .from(mediator_did.clone())
        .finalize())
    }
}
