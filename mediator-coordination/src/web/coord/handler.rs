use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use didcomm::Message;
use serde_json::json;
use uuid::Uuid;

use super::midlw::{self, *};
use crate::{
    constant::{
        DIDCOMM_ENCRYPTED_MIME_TYPE, MEDIATE_DENY_2_0, MEDIATE_GRANT_2_0, MEDIATE_REQUEST_2_0,
    },
    model::{
        coord::{MediationDeny, MediationGrant, MediationRequest, MediatorService},
        dic::{CompactDIC, DICPayload, JwtAssertable},
    },
    web::AppState,
};

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

    // Check explicit agreement to HTTP responding
    midlw::run!(ensure_transport_return_route_is_decorated_all(
        &plain_message
    ));

    // Attempt to parse message body into a mediation request
    let mediation_request = midlw::run!(parse_message_body_into_mediation_request(&plain_message));

    // Handle mediation request with its matching protocol
    let plain_response_message = midlw::run!(
        process_plain_mediation_request_over_dics(&state, &plain_message, &mediation_request).await
    );

    // Pack response message
    let packed_message = midlw::run!(
        pack_response_message(
            &plain_response_message,
            &state.did_resolver,
            &state.secrets_resolver
        )
        .await
    );

    // Build final response
    let response = (
        StatusCode::ACCEPTED,
        [(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)],
        Json(packed_message),
    );

    response.into_response()
}

/// Process a DIC-wise mediation request
pub async fn process_plain_mediation_request_over_dics(
    state: &AppState,
    plain_message: &Message,
    mediation_request: &MediationRequest,
) -> Result<Message, Response> {
    // Set convenient aliases
    let requester_did = &mediation_request.did;
    let mediator_did = &state.diddoc.id;

    // Check message type compliance
    midlw::ensure_mediation_request_type(mediation_request, MEDIATE_REQUEST_2_0)?;

    /* Deny mediate request if sender is not requester */

    let sender_did = plain_message
        .from
        .as_ref()
        .expect("should not panic as anonymous requests are rejected earlier");

    if sender_did != requester_did {
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
    }

    /* Issue mediate grant response */

    // Expand assertion key
    let (kid, jwk) = &state.assertion_jwk;

    // Issue verifiable credentials for DICs
    let vdic: Vec<_> = mediation_request
        .services
        .iter()
        .map(|service| {
            let dic = DICPayload {
                subject: requester_did.clone(),
                issuer: mediator_did.clone(),
                nonce: Some(Uuid::new_v4().to_string()),
                ..Default::default()
            };

            let jws = dic
                .sign(jwk, Some(kid.clone()))
                .expect("could not sign DIC payload");

            match service {
                MediatorService::Inbox => CompactDIC::Inbox(jws),
                MediatorService::Outbox => CompactDIC::Outbox(jws),
            }
        })
        .collect();

    let mediation_grant = MediationGrant {
        id: format!("urn:uuid:{}", Uuid::new_v4()),
        message_type: MEDIATE_GRANT_2_0.to_string(),
        endpoint: state.public_domain.to_string(),
        dic: vdic,
        ..Default::default()
    };

    Ok(Message::build(
        format!("urn:uuid:{}", Uuid::new_v4()),
        mediation_grant.message_type.clone(),
        json!(mediation_grant),
    )
    .to(requester_did.clone())
    .from(mediator_did.clone())
    .finalize())
}
