use didcomm::{Message, MessageBuilder};
use std::sync::Arc;
use uuid::Uuid;

use shared::state::AppState;

use crate::{constants::TRUST_PING_RESPONSE_2_0, error::TrustPingError, model::TrustPingResponse};

pub async fn handle_trust_ping(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, TrustPingError> {
    let mediator_did = &state.diddoc.id;
    let sender_did = message
        .from
        .as_ref()
        .ok_or(TrustPingError::MissingSenderDID)?;

    match message
        .body
        .get("response_requested")
        .and_then(|val| val.as_bool())
    {
        Some(true) => {
            let id = Uuid::new_v4().to_string();
            let response_builder: MessageBuilder = TrustPingResponse {
                id: id.as_str(),
                type_: TRUST_PING_RESPONSE_2_0,
                thid: message.id.as_str(),
            }
            .into();
            let response = response_builder
                .to(sender_did.to_owned())
                .from(mediator_did.to_owned())
                .finalize();
            Ok(Some(response))
        }
        Some(false) => Ok(None),
        None => Err(TrustPingError::MalformedRequest(
            "Missing \"response_requested\" field.".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::constants::TRUST_PING_2_0;
    use shared::utils::tests_utils::tests as global;

    #[tokio::test]
    async fn test_request_trust_ping_response() {
        let state = global::setup();

        let request = Message::build(
            "id_trust_ping".to_owned(),
            TRUST_PING_2_0.to_owned(),
            json!({"response_requested": true}),
        )
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let response = handle_trust_ping(Arc::clone(&state), request)
            .await
            .unwrap()
            .expect("Response should not be None");

        assert_eq!(response.type_, TRUST_PING_RESPONSE_2_0);
        assert_eq!(response.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(response.to.unwrap(), vec![global::_edge_did()]);
        assert_eq!(response.thid.unwrap(), "id_trust_ping");
    }

    #[tokio::test]
    async fn test_request_trust_ping_no_response() {
        let state = global::setup();

        let request = Message::build(
            "id_trust_ping".to_owned(),
            TRUST_PING_2_0.to_owned(),
            json!({"response_requested": false}),
        )
        .to(global::_mediator_did(&state))
        .from(global::_edge_did())
        .finalize();

        let response = handle_trust_ping(Arc::clone(&state), request)
            .await
            .unwrap();

        assert_eq!(response, None);
    }

    #[tokio::test]
    async fn test_request_trust_ping_error_with_missing_sender_did() {
        let state = global::setup();

        let request = Message::build(
            "id_trust_ping".to_owned(),
            TRUST_PING_2_0.to_owned(),
            json!({"response_requested": false}),
        )
        .to(global::_mediator_did(&state))
        .finalize();

        let response = handle_trust_ping(Arc::clone(&state), request)
            .await
            .unwrap_err();

        assert_eq!(response, TrustPingError::MissingSenderDID);
    }
}
