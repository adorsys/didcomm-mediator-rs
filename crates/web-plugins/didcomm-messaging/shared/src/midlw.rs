use crate::errors::MediationError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use didcomm::Message;
use serde_json::Value;

/// Validate explicit decoration on message to receive response on same route.
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

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::Value;

    pub async fn assert_error<T: Serialize + ?Sized>(
        response: Response,
        status: StatusCode,
        error: &T,
    ) {
        assert_eq!(response.status(), status);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_canon::to_string(&body).unwrap(),
            json_canon::to_string(error).unwrap()
        );
    }
}

#[cfg(test)]
mod midlw_test {
    use super::*;
    use crate::{
        constants::MEDIATE_REQUEST_2_0, midlw::tests::assert_error, utils::tests_utils::tests,
    };
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_ensure_transport_return_route_is_decorated_all() {
        let state = crate::utils::tests_utils::tests::setup();

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
                .to(tests::_mediator_did(&state))
                .from(tests::_edge_did())
            };
        }

        /* Positive cases */

        let msg = unfinalized_msg!()
            .header("return_route".into(), Value::String("all".into()))
            .finalize();

        assert!(ensure_transport_return_route_is_decorated_all(&msg).is_ok());

        /* Negative cases */

        let msg = unfinalized_msg!().finalize();

        assert_error(
            ensure_transport_return_route_is_decorated_all(&msg).unwrap_err(),
            StatusCode::BAD_REQUEST,
            &MediationError::NoReturnRouteAllDecoration.json().0,
        )
        .await;

        let msg = unfinalized_msg!()
            .header("return_route".into(), Value::String("none".into()))
            .finalize();

        assert_error(
            ensure_transport_return_route_is_decorated_all(&msg).unwrap_err(),
            StatusCode::BAD_REQUEST,
            &MediationError::NoReturnRouteAllDecoration.json().0,
        )
        .await;
    }
}
