use std::sync::Arc;

use super::midlw::{ensure_jwm_type_is_mediation_request, ensure_mediation_request_type};
use crate::{
    constants::{MEDIATE_DENY_DIC_1_0, MEDIATE_GRANT_DIC_1_0, MEDIATE_REQUEST_DIC_1_0},
    errors::MediationError,
    model::stateless::{
        coord::{MediationDeny, MediationGrant, MediationRequest, MediatorService},
        dic::{CompactDIC, DICPayload, JwtAssertable},
    },
};
use did_utils::didcore::VerificationMethodType;
use didcomm::Message;
use mongodb::bson::doc;
use serde_json::json;
use shared::{
    midlw::ensure_transport_return_route_is_decorated_all,
    state::{AppState, AppStateRepository},
};
use uuid::Uuid;

#[allow(dead_code)]
/// Process a DIC-wise mediation request
pub(crate) async fn process_plain_mediation_request_over_dics(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, MediationError> {
    // Convert to MediationRequest
    let mediation_request: MediationRequest =
        serde_json::from_value(json!(message)).map_err(|err| {
            tracing::error!("Failed to deserialize MediationRequest: {err:?}");
            MediationError::UnexpectedMessageFormat
        })?;

    // Set convenient aliases
    let requester_did = &mediation_request.did;
    let mediator_did = &state.diddoc.id;

    let AppStateRepository { keystore, .. } = state.repository.as_ref().ok_or_else(|| {
        tracing::error!("Missing persistence layer");
        MediationError::InternalServerError
    })?;

    // Check message type compliance
    ensure_mediation_request_type(&json!(mediation_request), MEDIATE_REQUEST_DIC_1_0)?;

    // Check message type compliance
    ensure_jwm_type_is_mediation_request(&message)?;

    // This is to Check explicit agreement to HTTP responding
    ensure_transport_return_route_is_decorated_all(&message)
        .map_err(|_| MediationError::NoReturnRouteAllDecoration)?;

    /* Deny mediate request if sender is not requester */

    let sender_did = message.from.as_ref().unwrap();

    if sender_did != requester_did {
        return Ok(Some(
            Message::build(
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
            .finalize(),
        ));
    }

    /* Issue mediate grant response */

    // Extract assertion key id
    let kid = state
        .diddoc
        .assertion_method
        .as_ref()
        .and_then(|vms| vms.first())
        .map(|vm| match vm {
            VerificationMethodType::Embedded(key) => &key.id,
            VerificationMethodType::Reference(id) => id,
        })
        .cloned()
        .ok_or_else(|| {
            tracing::error!("No assertion key found in DID document");
            MediationError::InternalServerError
        })?;

    // Extract assertion key
    let jwk = keystore
        .find_one_by(doc! { "kid": kid.clone() })
        .await
        .map_err(|err| {
            tracing::error!("Error fetching secret: {err:?}");
            MediationError::InternalServerError
        })?
        .ok_or_else(|| {
            tracing::error!("Secret not found");
            MediationError::InternalServerError
        })?
        .secret_material;

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
                .sign(&jwk, Some(kid.clone()))
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

    Ok(Some(
        Message::build(
            format!("urn:uuid:{}", Uuid::new_v4()),
            mediation_grant.message_type.clone(),
            json!(mediation_grant),
        )
        .to(requester_did.clone())
        .from(mediator_did.clone())
        .finalize(),
    ))
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use shared::utils::tests_utils::tests::*;

//     use crate::errors::MediationError;

//     #[tokio::test]
//     async fn test_comprehensive_mediation_grant_response() {
//         let state = setup();

//         // Build message
//         let msg = Message::build(
//             "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//             MEDIATE_REQUEST_DIC_1_0.to_string(),
//             json!(MediationRequest {
//                 id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
//                 message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
//                 did: _edge_did(),
//                 services: [MediatorService::Inbox, MediatorService::Outbox]
//                     .into_iter()
//                     .collect(),
//                 ..Default::default()
//             }),
//         )
//         .header("return-route".to_owned(), json!("all"))
//         .to(_mediator_did(&state))
//         .from(_edge_did())
//         .finalize();

//         // Process request
//         let response = process_plain_mediation_request_over_dics(Arc::clone(&state), msg)
//             .await
//             .unwrap()
//             .expect("Response should not be None");

//         // Assert metadata
//         assert_eq!(response.clone().type_, MEDIATE_GRANT_DIC_1_0);
//         assert_eq!(response.clone().from.unwrap(), _mediator_did(&state));
//         assert_eq!(response.clone().to.unwrap(), [_edge_did()]);

//         // Parse alleged mediation grant response
//         let mediation_grant: MediationGrant = serde_json::from_value(json!(response)).unwrap();

//         // Assert mediation grant's properties
//         assert!(mediation_grant.id.starts_with("urn:uuid:"));
//         assert_eq!(mediation_grant.message_type, MEDIATE_GRANT_DIC_1_0);
//         assert_eq!(mediation_grant.endpoint, state.public_domain);

//         // Assert that mediation grant embeds DICs for requested services
//         assert_eq!(
//             _inspect_dic_tags(&mediation_grant.dic),
//             vec!["inbox", "outbox"]
//         );

//         // Verify issued DICs
//         // assert!({
//         //     let mut iter = mediation_grant.dic.iter();
//         //     iter.all(|dic| {
//         //         jws::verify_compact_jws(&dic.plain_jws(), &state.assertion_jwk.1).is_ok()
//         //     })
//         // });
//     }

//     #[tokio::test]
//     async fn test_mediation_grant_response_with_single_channel() {
//         let state = setup();

//         for service in &[MediatorService::Inbox, MediatorService::Outbox] {
//             let msg = Message::build(
//                 "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//                 MEDIATE_REQUEST_DIC_1_0.to_string(),
//                 json!(MediationRequest {
//                     id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
//                     message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
//                     did: _edge_did(),
//                     services: [service.clone()].into_iter().collect(),
//                     ..Default::default()
//                 }),
//             )
//             .header("return-route".to_owned(), json!("all"))
//             .to(_mediator_did(&state))
//             .from(_edge_did())
//             .finalize();

//             let response = process_plain_mediation_request_over_dics(Arc::clone(&state), msg)
//                 .await
//                 .unwrap()
//                 .expect("Response should not be None");

//             let mediation_grant: MediationGrant = serde_json::from_value(json!(response)).unwrap();

//             assert_eq!(
//                 _inspect_dic_tags(&mediation_grant.dic),
//                 vec![match service {
//                     MediatorService::Inbox => "inbox",
//                     MediatorService::Outbox => "outbox",
//                 }]
//             );
//         }
//     }

//     #[tokio::test]
//     async fn test_mediation_deny_response_for_sender_requester_unmatch() {
//         let state = setup();

//         let msg = Message::build(
//             "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//             MEDIATE_REQUEST_DIC_1_0.to_string(),
//             json!(MediationRequest {
//                 id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
//                 message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
//                 did: "did:key:unknown".to_string(),
//                 services: [MediatorService::Inbox, MediatorService::Outbox]
//                     .into_iter()
//                     .collect(),
//                 ..Default::default()
//             }),
//         )
//         .header("return-route".into(), json!("all"))
//         .to(_mediator_did(&state))
//         .from(_edge_did())
//         .finalize();

//         let response = process_plain_mediation_request_over_dics(Arc::clone(&state), msg)
//             .await
//             .unwrap()
//             .expect("Response should not be None");

//         assert_eq!(response.clone().type_, MEDIATE_DENY_DIC_1_0);
//         assert_eq!(response.clone().from.unwrap(), _mediator_did(&state));
//         assert_eq!(response.to.clone().unwrap(), [_edge_did()]);

//         let mediation_deny: MediationDeny = serde_json::from_value(json!(response)).unwrap();
//         assert_eq!(mediation_deny.message_type, MEDIATE_DENY_DIC_1_0);
//     }

// #[tokio::test]
// async fn test_bad_request_on_invalid_payload_content_type() {
//     let state = setup();

//     let msg = Message::build(
//         "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//         MEDIATE_REQUEST_DIC_1_0.to_string(),
//         json!(MediationRequest {
//             id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
//             message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
//             did: _edge_did(),
//             services: [MediatorService::Inbox, MediatorService::Outbox]
//                 .into_iter()
//                 .collect(),
//             ..Default::default()
//         }),
//     )
//     .header("return-route".into(), json!("all"))
//     .to(_mediator_did(&state))
//     .from(_edge_did())
//     .finalize();

//     let response = process_plain_mediation_request_over_dics(Arc::clone(&state), msg).await;

//     assert_eq!(
//         response.unwrap_err(),
//         MediationError::NotDidcommEncryptedPayload
//     )
// }

// #[tokio::test]
// async fn test_bad_request_on_unpacking_failure() {
//     let (app, state) = setup();

//     let msg = Message::build(
//         "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//         MEDIATE_REQUEST_DIC_1_0.to_string(),
//         json!(MediationRequest {
//             id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
//             message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
//             did: _edge_did(),
//             services: [MediatorService::Inbox, MediatorService::Outbox]
//                 .into_iter()
//                 .collect(),
//             ..Default::default()
//         }),
//     )
//     .header("return-route".into(), json!("all"))
//     .to(_edge_did())
//     .from(_edge_did())
//     .finalize();

//     // Pack for edge instead of mediator
//     let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _edge_did())
//         .await
//         .unwrap();

//     // Send request
//     let response = app
//         .oneshot(
//             Request::builder()
//                 .uri(String::from("/"))
//                 .method(Method::POST)
//                 .header(CONTENT_TYPE, "application/didcomm-encrypted+json")
//                 .body(Body::from(packed_msg))
//                 .unwrap(),
//         )
//         .await
//         .unwrap();

//     assert_eq!(response.status(), StatusCode::BAD_REQUEST);
//     assert_eq!(
//         response.headers().get(CONTENT_TYPE).unwrap(),
//         "application/json"
//     );

//     let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
//     let body: Value = serde_json::from_slice(&body).unwrap();

//     assert_eq!(
//         json_canon::to_string(&body).unwrap(),
//         json_canon::to_string(&MediationError::MessageUnpackingFailure.json().0).unwrap()
//     )
// }

// #[tokio::test]
// async fn test_bad_request_on_invalid_jwm_header_message_type() {
//     let (app, state) = setup();

//     let msg = Message::build(
//         "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//         "invalid-message-type".to_string(),
//         json!(MediationRequest {
//             id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
//             message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
//             did: _edge_did(),
//             services: [MediatorService::Inbox, MediatorService::Outbox]
//                 .into_iter()
//                 .collect(),
//             ..Default::default()
//         }),
//     )
//     .header("return-route".into(), json!("all"))
//     .to(_mediator_did(&state))
//     .from(_edge_did())
//     .finalize();

//     let packed_msg = _edge_pack_message(&state, &msg, Some(_edge_did()), _mediator_did(&state))
//         .await
//         .unwrap();

//     // Send request
//     let response = app
//         .oneshot(
//             Request::builder()
//                 .uri(String::from("/"))
//                 .method(Method::POST)
//                 .header(CONTENT_TYPE, "application/didcomm-encrypted+json")
//                 .body(Body::from(packed_msg))
//                 .unwrap(),
//         )
//         .await
//         .unwrap();

//     assert_eq!(response.status(), StatusCode::BAD_REQUEST);
//     assert_eq!(
//         response.headers().get(CONTENT_TYPE).unwrap(),
//         "application/json"
//     );

//     let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
//     let body: Value = serde_json::from_slice(&body).unwrap();

//     assert_eq!(
//         json_canon::to_string(&body).unwrap(),
//         json_canon::to_string(&MediationError::InvalidMessageType.json().0).unwrap()
//     )
// }

//     #[tokio::test]
//     async fn test_bad_request_on_missing_return_route_decoration() {
//         let state = setup();

//         let msg = Message::build(
//             "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//             MEDIATE_REQUEST_DIC_1_0.to_string(),
//             json!(MediationRequest {
//                 id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
//                 message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
//                 did: _edge_did(),
//                 services: [MediatorService::Inbox, MediatorService::Outbox]
//                     .into_iter()
//                     .collect(),
//                 ..Default::default()
//             }),
//         )
//         .to(_mediator_did(&state))
//         .from(_edge_did())
//         .finalize();

//         let response = process_plain_mediation_request_over_dics(Arc::clone(&state), msg).await;

//         assert_eq!(
//             response.unwrap_err(),
//             MediationError::NoReturnRouteAllDecoration
//         )
//     }

//     #[tokio::test]
//     async fn test_bad_request_on_non_mediate_request_payload() {
//         let state = setup();

//         let msg = Message::build(
//             "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//             MEDIATE_REQUEST_DIC_1_0.to_string(),
//             json!("not-mediate-request"),
//         )
//         .header("return-route".into(), json!("all"))
//         .to(_mediator_did(&state))
//         .from(_edge_did())
//         .finalize();

//         let response = process_plain_mediation_request_over_dics(Arc::clone(&state), msg).await;

//         assert_eq!(
//             response.unwrap_err(),
//             MediationError::UnexpectedMessageFormat
//         )
//     }

//     //------------------------------------------------------------------------
//     // Helpers ---------------------------------------------------------------
//     //------------------------------------------------------------------------

//     fn _inspect_dic_tags(vdic: &[CompactDIC]) -> Vec<&'static str> {
//         let mut tags: Vec<_> = vdic
//             .iter()
//             .map(|dic| match dic {
//                 CompactDIC::Inbox(_) => "inbox",
//                 CompactDIC::Outbox(_) => "outbox",
//             })
//             .collect();

//         tags.sort();
//         tags
//     }
// }
