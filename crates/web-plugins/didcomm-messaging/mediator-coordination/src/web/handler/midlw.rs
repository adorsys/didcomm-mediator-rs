use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use didcomm::Message;
use serde_json::{json, Value};

use shared::{
    constants::{MEDIATE_REQUEST_2_0, MEDIATE_REQUEST_DIC_1_0},
    errors::MediationError,
};

macro_rules! run {
    ($expression:expr) => {
        match $expression {
            Ok(res) => res,

            Err(err) => return Err(err),
        }
    };
}

pub(crate) use run;

/// Validate that JWM's indicative body type is a mediation request
pub(crate) fn ensure_jwm_type_is_mediation_request(message: &Message) -> Result<(), Response> {
    if ![MEDIATE_REQUEST_2_0, MEDIATE_REQUEST_DIC_1_0].contains(&message.type_.as_str()) {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType.json(),
        );

        return Err(response.into_response());
    }

    Ok(())
}

/// Validate explicit decoration on message to receive response on same route
/// See https://github.com/hyperledger/aries-rfcs/tree/main/features/0092-transport-return-route
pub fn ensure_transport_return_route_is_decorated_all(message: &Message) -> Result<(), Response> {
    if message
        .extra_headers
        .get("return_route")
        .and_then(Value::as_str)
        != Some("all")
    {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::NoReturnRouteAllDecoration.json(),
        );

        return Err(response.into_response());
    }

    Ok(())
}

/// Validate that mediation request's URI type is as expected
#[allow(dead_code)]
pub fn ensure_mediation_request_type(
    mediation_request: &Value,
    message_type: &str,
) -> Result<(), Response> {
    if mediation_request.get("type") != Some(&json!(message_type)) {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType.json(),
        );

        return Err(response.into_response());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::utils::tests_utils::tests::*;

    #[cfg(feature = "stateless")]
    use crate::model::stateless::coord::{
        MediationRequest as StatelessMediationRequest, MediatorService,
    };

    use didcomm::Message;

    #[tokio::test]
    async fn test_ensure_jwm_type_is_mediation_request() {
        let state = setup();

        /* Positive cases */
        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_2_0.to_string(),
            json!({
                "id": "id_alice_mediation_request",
                "type": MEDIATE_REQUEST_2_0,
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
                "id": "id_alice_mediation_request",
                "type": MEDIATE_REQUEST_DIC_1_0,
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
                "id": "id_alice_mediation_request",
                "type": MEDIATE_REQUEST_2_0,
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        _assert_midlw_err(
            ensure_jwm_type_is_mediation_request(&msg).unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType,
        )
        .await;
    }

    #[tokio::test]
    async fn test_ensure_transport_return_route_is_decorated_all() {
        let state = setup();

        macro_rules! unfinalized_msg {
            () => {
                Message::build(
                    "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
                    MEDIATE_REQUEST_2_0.to_string(),
                    json!({
                        "id": "id_alice_mediation_request",
                        "type": MEDIATE_REQUEST_2_0,
                        "did": "did:key:alice_identity_pub@alice_mediator",
                        "services": ["inbox", "outbox"]
                    }),
                )
                .to(_mediator_did(&state))
                .from(_edge_did())
            };
        }

        /* Positive cases */

        let msg = unfinalized_msg!()
            .header("return_route".into(), Value::String("all".into()))
            .finalize();

        assert!(ensure_transport_return_route_is_decorated_all(&msg).is_ok());

        /* Negative cases */

        let msg = unfinalized_msg!().finalize();

        _assert_midlw_err(
            ensure_transport_return_route_is_decorated_all(&msg).unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::NoReturnRouteAllDecoration,
        )
        .await;

        let msg = unfinalized_msg!()
            .header("return_route".into(), Value::String("none".into()))
            .finalize();

        _assert_midlw_err(
            ensure_transport_return_route_is_decorated_all(&msg).unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::NoReturnRouteAllDecoration,
        )
        .await;
    }

    #[cfg(feature = "stateless")]
    #[tokio::test]
    async fn test_parse_message_body_into_stateless_mediation_request() {
        let (_, state) = setup();

        /* Positive cases */

        let mediation_request = StatelessMediationRequest {
            id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
            message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
            did: _edge_did(),
            services: [MediatorService::Inbox, MediatorService::Outbox]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!(&mediation_request),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        let parsed_mediation_request = parse_message_body_into_mediation_request(&msg).unwrap();
        #[allow(irrefutable_let_patterns)]
        let MediationRequest::Stateless(parsed_mediation_request) = parsed_mediation_request
        else {
            panic!()
        };
        assert_eq!(json!(mediation_request), json!(parsed_mediation_request));

        /* Negative cases */

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            "invalid-type".to_string(),
            json!("not-mediation-request"),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        _assert_midlw_err(
            parse_message_body_into_mediation_request(&msg).unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::UnexpectedMessageFormat,
        )
        .await;
    }

    #[cfg(feature = "stateless")]
    #[tokio::test]
    async fn test_ensure_mediation_request_type() {
        /* Positive cases */

        let mediation_request = StatelessMediationRequest {
            id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
            message_type: MEDIATE_REQUEST_DIC_1_0.to_string(),
            did: _edge_did(),
            services: [MediatorService::Inbox, MediatorService::Outbox]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        assert!(
            ensure_mediation_request_type(&json!(mediation_request), MEDIATE_REQUEST_DIC_1_0)
                .is_ok()
        );
        _assert_midlw_err(
            ensure_mediation_request_type(&json!(mediation_request), MEDIATE_REQUEST_2_0)
                .unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType,
        )
        .await;

        /* Negative cases */

        let mediation_request = StatelessMediationRequest {
            id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
            message_type: "invalid-type".to_string(),
            did: _edge_did(),
            services: [MediatorService::Inbox, MediatorService::Outbox]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        _assert_midlw_err(
            ensure_mediation_request_type(&json!(mediation_request), MEDIATE_REQUEST_DIC_1_0)
                .unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType,
        )
        .await;
    }

    //----------------------------------------------------------------------------------------------
    // Helpers -------------------------------------------------------------------------------------
    //----------------------------------------------------------------------------------------------

    async fn _assert_midlw_err(err: Response, status: StatusCode, mediation_error: MediationError) {
        assert_eq!(err.status(), status);

        let body = hyper::body::to_bytes(err.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_canon::to_string(&body).unwrap(),
            json_canon::to_string(&mediation_error.json().0).unwrap()
        );
    }
}
