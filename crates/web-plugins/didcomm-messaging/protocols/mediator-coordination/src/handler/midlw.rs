use crate::{
    constants::{MEDIATE_REQUEST_2_0, MEDIATE_REQUEST_DIC_1_0},
    errors::MediationError,
};
use didcomm::Message;
use serde_json::{json, Value};
/// Validate that JWM's indicative body type is a mediation request
pub(crate) fn ensure_jwm_type_is_mediation_request(
    message: &Message,
) -> Result<(), MediationError> {
    if ![MEDIATE_REQUEST_2_0, MEDIATE_REQUEST_DIC_1_0].contains(&message.type_.as_str()) {
        return Err(MediationError::InvalidMessageType);
    }

    Ok(())
}

/// Validate that mediation request's URI type is as expected
#[allow(dead_code)]
pub fn ensure_mediation_request_type(
    mediation_request: &Value,
    message_type: &str,
) -> Result<(), MediationError> {
    if mediation_request.get("type") != Some(&json!(message_type)) {
        return Err(MediationError::InvalidMessageType);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::utils::tests_utils::tests::*;

    // #[cfg(feature = "stateless")]
    // use crate::model::stateless::coord::{
    //     MediationRequest as StatelessMediationRequest, MediatorService,
    // };

    use didcomm::Message;

    #[tokio::test]
    async fn test_ensure_jwm_type_is_mediation_request() {
        let state = setup();

        /* Positive cases */
        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_2_0.to_string(),
            json!({
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        assert!(ensure_jwm_type_is_mediation_request(&msg).is_ok());

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!({
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        assert!(ensure_jwm_type_is_mediation_request(&msg).is_ok());

        /* Negative cases */

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            "invalid-type".to_string(),
            json!({
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        assert_eq!(
            ensure_jwm_type_is_mediation_request(&msg).unwrap_err(),
            MediationError::InvalidMessageType,
        );
    }

    // #[cfg(feature = "stateless")]
    //     #[tokio::test]
    //     async fn test_parse_message_body_into_stateless_mediation_request() {
    //         let state = setup();

    //         /* Positive cases */

    //         let mediation_request = StatelessMediationRequest {
    //             id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
    //             message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
    //             did: _edge_did(),
    //             services: [MediatorService::Inbox, MediatorService::Outbox]
    //                 .into_iter()
    //                 .collect(),
    //             ..Default::default()
    //         };

    //         let msg = Message::build(
    //             "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
    //             MEDIATE_REQUEST_DIC_1_0.to_string(),
    //             json!(&mediation_request),
    //         )
    //         .to(_mediator_did(&state))
    //         .from(_edge_did())
    //         .finalize();

    //         let parsed_mediation_request = parse_message_body_into_mediation_request(&msg).unwrap();
    //         #[allow(irrefutable_let_patterns)]
    //         let MediationRequest::Stateless(parsed_mediation_request) = parsed_mediation_request
    //         else {
    //             panic!()
    //         };
    //         assert_eq!(json!(mediation_request), json!(parsed_mediation_request));

    //         /* Negative cases */

    //         let msg = Message::build(
    //             "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
    //             "invalid-type".to_string(),
    //             json!("not-mediation-request"),
    //         )
    //         .to(_mediator_did(&state))
    //         .from(_edge_did())
    //         .finalize();

    //         assert_error(
    //             parse_message_body_into_mediation_request(&msg).unwrap_err(),
    //             StatusCode::BAD_REQUEST,
    //             &MediationError::UnexpectedMessageFormat.json().0,
    //         )
    //         .await;
    //     }

    //     // #[cfg(feature = "stateless")]
    //     #[tokio::test]
    //     async fn test_ensure_mediation_request_type() {
    //         /* Positive cases */

    //         let mediation_request = StatelessMediationRequest {
    //             id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
    //             message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
    //             did: _edge_did(),
    //             services: [MediatorService::Inbox, MediatorService::Outbox]
    //                 .into_iter()
    //                 .collect(),
    //             ..Default::default()
    //         };

    //         assert!(
    //             ensure_mediation_request_type(&json!(mediation_request), MEDIATE_REQUEST_DIC_1_0)
    //                 .is_ok()
    //         );
    //         assert_error(
    //             ensure_mediation_request_type(&json!(mediation_request), MEDIATE_REQUEST_2_0)
    //                 .unwrap_err(),
    //             StatusCode::BAD_REQUEST,
    //             &MediationError::InvalidMessageType.json().0,
    //         )
    //         .await;

    //         /* Negative cases */

    //         let mediation_request = StatelessMediationRequest {
    //             id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
    //             message_type: "invalid-type".to_string(),
    //             did: _edge_did(),
    //             services: [MediatorService::Inbox, MediatorService::Outbox]
    //                 .into_iter()
    //                 .collect(),
    //             ..Default::default()
    //         };

    //         assert_error(
    //             ensure_mediation_request_type(&json!(mediation_request), MEDIATE_REQUEST_DIC_1_0)
    //                 .unwrap_err(),
    //             StatusCode::BAD_REQUEST,
    //             &MediationError::InvalidMessageType.json().0,
    //         )
    //         .await;
    //     }
}
