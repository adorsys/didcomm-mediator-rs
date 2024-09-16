use axum::response::Response;
use didcomm::Message;
use serde_json::json;
use uuid::Uuid;

use super::midlw::{self};
use crate::{
    constant::{MEDIATE_DENY_DIC_1_0, MEDIATE_GRANT_DIC_1_0, MEDIATE_REQUEST_DIC_1_0},
    model::stateless::{
        coord::{MediationDeny, MediationGrant, MediationRequest, MediatorService},
        dic::{CompactDIC, DICPayload, JwtAssertable},
    },
    web::AppState,
};

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
    midlw::ensure_mediation_request_type(&json!(mediation_request), MEDIATE_REQUEST_DIC_1_0)?;

    // Check message type compliance
    midlw::run!(ensure_jwm_type_is_mediation_request(&plain_message));

    // Check explicit agreement to HTTP responding
    midlw::run!(ensure_transport_return_route_is_decorated_all(
        &plain_message
    ));

    /* Deny mediate request if sender is not requester */

    let sender_did = plain_message
        .from
        .as_ref()
        .expect("should not panic as anonymous requests are rejected earlier");

    if sender_did != requester_did {
        return Ok(Message::build(
            format!("urn:uuid:{}", Uuid::new_v4()),
            MEDIATE_DENY_DIC_1_0.to_string(),
            json!(MediationDeny {
                id: format!("urn:uuid:{}", Uuid::new_v4()),
                message_type: MEDIATE_DENY_DIC_1_0.to_string(),
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
        message_type: MEDIATE_GRANT_DIC_1_0.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::Body,
        http::{header::CONTENT_TYPE, Method, Request},
    };
    use hyper::StatusCode;
    use serde_json::Value;
    use tower::util::ServiceExt;

    use crate::{
        constant::DIDCOMM_ENCRYPTED_MIME_TYPE,
        jose::jws,
        web::{error::MediationError, handler::tests::*},
    };

    #[tokio::test]
    async fn test_comprehensive_mediation_grant_response() {
        let (app, state) = setup();

        // Build message
        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
                did: _edge_did(),
                services: [MediatorService::Inbox, MediatorService::Outbox]
                    .into_iter()
                    .collect(),
                ..Default::default()
            }),
        )
        .header(
            "~transport".into(),
            json!({
                "return_route": "all"
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        // Encrypt message for mediator
        let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _mediator_did(&state))
            .await
            .unwrap();

        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert response's metadata
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            DIDCOMM_ENCRYPTED_MIME_TYPE
        );

        // Parse response's body
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        let response = serde_json::to_string(&body).unwrap();

        // Decrypt response
        let msg: Message = _edge_unpack_message(&state, &response).await.unwrap();

        // Assert metadata
        assert_eq!(msg.type_, MEDIATE_GRANT_DIC_1_0);
        assert_eq!(msg.from.unwrap(), _mediator_did(&state));
        assert_eq!(msg.to.unwrap(), [_edge_did()]);

        // Parse alleged mediation grant response
        let mediation_grant: MediationGrant = serde_json::from_value(msg.body).unwrap();

        // Assert mediation grant's properties
        assert!(mediation_grant.id.starts_with("urn:uuid:"));
        assert_eq!(mediation_grant.message_type, MEDIATE_GRANT_DIC_1_0);
        assert_eq!(mediation_grant.endpoint, state.public_domain);

        // Assert that mediation grant embeds DICs for requested services
        assert_eq!(
            _inspect_dic_tags(&mediation_grant.dic),
            vec!["inbox", "outbox"]
        );

        // Verify issued DICs
        assert!({
            let mut iter = mediation_grant.dic.iter();
            iter.all(|dic| {
                jws::verify_compact_jws(&dic.plain_jws(), &state.assertion_jwk.1).is_ok()
            })
        });
    }

    #[tokio::test]
    async fn test_mediation_grant_response_with_single_channel() {
        let (app, state) = setup();

        for service in &[MediatorService::Inbox, MediatorService::Outbox] {
            let msg = Message::build(
                "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
                MEDIATE_REQUEST_DIC_1_0.to_string(),
                json!(MediationRequest {
                    id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                    message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
                    did: _edge_did(),
                    services: [service.clone()].into_iter().collect(),
                    ..Default::default()
                }),
            )
            .header(
                "~transport".into(),
                json!({
                    "return_route": "all"
                }),
            )
            .to(_mediator_did(&state))
            .from(_edge_did())
            .finalize();

            let packed_msg =
                _edge_pack_message(&state, &msg, Some(_edge_did()), _mediator_did(&state))
                    .await
                    .unwrap();

            // Send request
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(String::from("/mediate"))
                        .method(Method::POST)
                        .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                        .body(Body::from(packed_msg))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::ACCEPTED);
            assert_eq!(
                response.headers().get(CONTENT_TYPE).unwrap(),
                DIDCOMM_ENCRYPTED_MIME_TYPE
            );

            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body: Value = serde_json::from_slice(&body).unwrap();
            let response = serde_json::to_string(&body).unwrap();

            let msg: Message = _edge_unpack_message(&state, &response).await.unwrap();
            let mediation_grant: MediationGrant = serde_json::from_value(msg.body).unwrap();

            assert_eq!(
                _inspect_dic_tags(&mediation_grant.dic),
                vec![match service {
                    MediatorService::Inbox => "inbox",
                    MediatorService::Outbox => "outbox",
                }]
            );
        }
    }

    #[tokio::test]
    async fn test_mediation_deny_response_for_sender_requester_unmatch() {
        let (app, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
                did: "did:key:unknown".to_string(),
                services: [MediatorService::Inbox, MediatorService::Outbox]
                    .into_iter()
                    .collect(),
                ..Default::default()
            }),
        )
        .header(
            "~transport".into(),
            json!({
                "return_route": "all"
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _mediator_did(&state))
            .await
            .unwrap();

        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            DIDCOMM_ENCRYPTED_MIME_TYPE
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        let response = serde_json::to_string(&body).unwrap();

        let msg: Message = _edge_unpack_message(&state, &response).await.unwrap();
        assert_eq!(msg.type_, MEDIATE_DENY_DIC_1_0);
        assert_eq!(msg.from.unwrap(), _mediator_did(&state));
        assert_eq!(msg.to.unwrap(), [_edge_did()]);

        let mediation_deny: MediationDeny = serde_json::from_value(msg.body).unwrap();
        assert_eq!(mediation_deny.message_type, MEDIATE_DENY_DIC_1_0);
    }

    #[tokio::test]
    async fn test_bad_request_on_invalid_payload_content_type() {
        let (app, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
                did: _edge_did(),
                services: [MediatorService::Inbox, MediatorService::Outbox]
                    .into_iter()
                    .collect(),
                ..Default::default()
            }),
        )
        .header(
            "~transport".into(),
            json!({
                "return_route": "all"
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _mediator_did(&state))
            .await
            .unwrap();

        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_canon::to_string(&body).unwrap(),
            json_canon::to_string(&MediationError::NotDidcommEncryptedPayload.json().0).unwrap()
        )
    }

    #[tokio::test]
    async fn test_bad_request_on_unpacking_failure() {
        let (app, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
                did: _edge_did(),
                services: [MediatorService::Inbox, MediatorService::Outbox]
                    .into_iter()
                    .collect(),
                ..Default::default()
            }),
        )
        .header(
            "~transport".into(),
            json!({
                "return_route": "all"
            }),
        )
        .to(_edge_did())
        .from(_edge_did())
        .finalize();

        // Pack for edge instead of mediator
        let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _edge_did())
            .await
            .unwrap();

        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_canon::to_string(&body).unwrap(),
            json_canon::to_string(&MediationError::MessageUnpackingFailure.json().0).unwrap()
        )
    }

    #[tokio::test]
    async fn test_bad_request_on_invalid_jwm_header_message_type() {
        let (app, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            "invalid-message-type".to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
                did: _edge_did(),
                services: [MediatorService::Inbox, MediatorService::Outbox]
                    .into_iter()
                    .collect(),
                ..Default::default()
            }),
        )
        .header(
            "~transport".into(),
            json!({
                "return_route": "all"
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _mediator_did(&state))
            .await
            .unwrap();

        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_canon::to_string(&body).unwrap(),
            json_canon::to_string(&MediationError::InvalidMessageType.json().0).unwrap()
        )
    }

    #[tokio::test]
    async fn test_bad_request_on_missing_return_route_decoration() {
        let (app, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
                did: _edge_did(),
                services: [MediatorService::Inbox, MediatorService::Outbox]
                    .into_iter()
                    .collect(),
                ..Default::default()
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _mediator_did(&state))
            .await
            .unwrap();
        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_canon::to_string(&body).unwrap(),
            json_canon::to_string(&MediationError::NoReturnRouteAllDecoration.json().0).unwrap()
        )
    }

    #[tokio::test]
    async fn test_bad_request_on_non_mediate_request_payload() {
        let (app, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!("not-mediate-request"),
        )
        .header(
            "~transport".into(),
            json!({
                "return_route": "all"
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _mediator_did(&state))
            .await
            .unwrap();

        // Send request
        let response = app
            .oneshot(
                Request::builder()
                    .uri(String::from("/mediate"))
                    .method(Method::POST)
                    .header(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE)
                    .body(Body::from(packed_msg))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_canon::to_string(&body).unwrap(),
            json_canon::to_string(&MediationError::UnexpectedMessageFormat.json().0).unwrap()
        )
    }

    //------------------------------------------------------------------------
    // Helpers ---------------------------------------------------------------
    //------------------------------------------------------------------------

    fn _inspect_dic_tags(vdic: &[CompactDIC]) -> Vec<&'static str> {
        let mut tags: Vec<_> = vdic
            .iter()
            .map(|dic| match dic {
                CompactDIC::Inbox(_) => "inbox",
                CompactDIC::Outbox(_) => "outbox",
            })
            .collect();

        tags.sort();
        tags
    }
}
