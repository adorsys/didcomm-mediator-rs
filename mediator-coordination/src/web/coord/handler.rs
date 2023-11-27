use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use didcomm::Message;
use serde_json::json;
use uuid::Uuid;

use super::midlw::{self, *};
use crate::{constant::{MEDIATE_REQUEST_2_0, MEDIATE_GRANT_2_0}, model::coord::{MediationRequest, MediationGrant}, web::AppState};

#[axum::debug_handler]
pub async fn process_didcomm_mediation_request_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: String,
) -> Response {
    // Enforce request content type to match `didcomm-encrypted+json`
    midlw::run!(ensure_content_type_is_didcomm_encrypted(&headers));

    // Unpack payload message
    let plain_message = midlw::run!(
        unpack_request_message(&payload, &state.did_resolver, &state.secrets_resolver).await
    );

    // Check message type compliance
    midlw::run!(ensure_jwm_type_is_mediation_request(&plain_message));

    // Attempt to parse message body into a mediation request
    let mediation_request = midlw::run!(parse_message_body_into_mediation_request(&plain_message));

    (StatusCode::ACCEPTED, Json(json!(mediation_request))).into_response()
}

/// Process a DIC-wise mediation request
pub async fn process_plain_mediation_request_over_dics(
    state: &AppState,
    plain_message: &Message,
    mediation_request: &MediationRequest,
) -> Result<Message, Response> {
    // midlw::run!(ensure_mediation_request_type(
    //     mediation_request,
    //     MEDIATE_REQUEST_2_0
    // ));

    // Issue mediate grant response

    let mediation_grant = MediationGrant {
        id: mediation_request.id.clone(),
        message_type: MEDIATE_GRANT_2_0.to_string(),
        endpoint: state.public_domain.to_string(),
        dic: vec![],
        ..Default::default()
    };

    Ok(Message::build(
        format!("urn:uuid:{}", Uuid::new_v4()),
        mediation_grant.message_type.clone(),
        json!(mediation_grant),
    )
    .to(mediation_request.did.clone())
    .from(state.diddoc.id.clone())
    .finalize())
}
