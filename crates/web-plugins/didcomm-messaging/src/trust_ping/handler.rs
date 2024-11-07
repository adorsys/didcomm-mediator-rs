use axum::response::{IntoResponse, Response};
use didcomm::{Message, MessageBuilder};
use hyper::StatusCode;
use std::sync::Arc;
use uuid::Uuid;

use crate::web::AppState;

use super::{constants::TRUST_PING_RESPONSE_2_0, error::TrustPingError, model::TrustPingResponse};

pub(crate) async fn handle_trust_ping(
    state: Arc<AppState>,
    message: Message,
) -> Result<Message, Response> {
    let mediator_did = &state.diddoc.id;
    let sender_did = message
        .from
        .as_ref()
        .ok_or(TrustPingError::MissingSenderDID.into_response())?;

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
            Ok(response)
        }
        Some(false) => Err(StatusCode::OK.into_response()),
        None => Err(
            TrustPingError::MalformedRequest("Missing \"response_requested\" field.")
                .into_response(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::*;
    use crate::{trust_ping::constants::TRUST_PING_2_0, web::handler::tests as global};

    async fn assert_error(
        response: Response,
        status: StatusCode,
        error: Option<TrustPingError<'_>>,
    ) {
        assert_eq!(response.status(), status);

        if let Some(error) = error {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body: Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(
                json_canon::to_string(&body).unwrap(),
                json_canon::to_string(&json!({"error": error.to_string()})).unwrap()
            );
        }
    }

    #[tokio::test]
    async fn test_request_trust_ping_response() {
        let (_, state) = global::setup();

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
            .unwrap();

        assert_eq!(response.type_, TRUST_PING_RESPONSE_2_0);
        assert_eq!(response.from.unwrap(), global::_mediator_did(&state));
        assert_eq!(response.to.unwrap(), vec![global::_edge_did()]);
        assert_eq!(response.thid.unwrap(), "id_trust_ping");
    }

    #[tokio::test]
    async fn test_request_trust_ping_no_response() {
        let (_, state) = global::setup();

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
            .unwrap_err();

        assert_error(response, StatusCode::OK, None).await;
    }

    #[tokio::test]
    async fn test_request_trust_ping_error_with_missing_sender_did() {
        let (_, state) = global::setup();

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

        assert_error(response, StatusCode::BAD_REQUEST, Some(TrustPingError::MissingSenderDID)).await;
    }
}
