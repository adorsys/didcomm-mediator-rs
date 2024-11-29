use crate::errors::SharedError;
use didcomm::Message;
use serde_json::Value;

/// Validate explicit decoration on message to receive response on same route.
pub fn ensure_transport_return_route_is_decorated_all(
    message: &Message,
) -> Result<(), SharedError> {
    if message
        .extra_headers
        .get("return_route")
        .and_then(Value::as_str)
        != Some("all")
    {
        return Err(SharedError::NoReturnRouteAllDecoration);
    }

    Ok(())
}

#[cfg(test)]
mod midlw_test {
    use super::*;
    use crate::utils::tests_utils::tests;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_ensure_transport_return_route_is_decorated_all() {
        let state = crate::utils::tests_utils::tests::setup();

        macro_rules! unfinalized_msg {
            () => {
                Message::build(
                    "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
                    "https://didcomm.org/coordinate-mediation/2.0/mediate-request".to_string(),
                    json!({
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

        assert_eq!(
            ensure_transport_return_route_is_decorated_all(&msg).unwrap_err(),
            SharedError::NoReturnRouteAllDecoration
        );

        let msg = unfinalized_msg!()
            .header("return_route".into(), Value::String("none".into()))
            .finalize();

        assert_eq!(
            ensure_transport_return_route_is_decorated_all(&msg).unwrap_err(),
            SharedError::NoReturnRouteAllDecoration
        );
    }
}
