mod stateless;
use stateless::process_plain_mediation_request_over_dics;

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::web::AppState;
use crate::{
    constant::DIDCOMM_ENCRYPTED_MIME_TYPE,
    model::coord::MediationRequest,
    web::coord::midlw::{self, *},
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
    let plain_response_message = match mediation_request {
        MediationRequest::Stateless(req) => midlw::run!(
            process_plain_mediation_request_over_dics(&state, &plain_message, &req).await
        ),
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::Body,
        http::{header::CONTENT_TYPE, Method, Request},
        Router,
    };
    use did_utils::key_jwk::jwk::Jwk;
    use didcomm::{
        error::Error as DidcommError, secrets::SecretsResolver, Message, PackEncryptedOptions,
        UnpackOptions,
    };
    use hyper::StatusCode;
    use serde_json::{json, Value};
    use tower::util::ServiceExt;

    use crate::{
        constant::{DIDCOMM_ENCRYPTED_MIME_TYPE, MEDIATE_REQUEST_2_0},
        didcomm::bridge::LocalSecretsResolver,
        model::stateless::{
            coord::{MediationRequest, MediatorService},
            dic::CompactDIC,
        },
        util::{self, MockFileSystem},
        web::{self, coord::error::MediationError},
    };

    pub fn setup() -> (Router, AppState) {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let mut mock_fs = MockFileSystem;
        let keystore_clone = util::read_keystore(&mut mock_fs, "").unwrap();

        let app = web::routes(public_domain.clone(), diddoc.clone(), keystore_clone);
        let state = AppState::from(public_domain, diddoc, keystore);

        (app, state)
    }

    #[tokio::test]
    async fn test_bad_request_on_invalid_payload_content_type() {
        let (app, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_2_0.to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_2_0.to_string(),
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
            MEDIATE_REQUEST_2_0.to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_2_0.to_string(),
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
    async fn test_bad_request_on_invalid_message_type() {
        let (app, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            "invalid-message-type".to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_2_0.to_string(),
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
            MEDIATE_REQUEST_2_0.to_string(),
            json!(MediationRequest {
                id: "urn:uuid:ff5a4c85-0df4-4fbe-88ce-fcd2d321a06d".to_string(),
                message_type: MEDIATE_REQUEST_2_0.to_string(),
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
            MEDIATE_REQUEST_2_0.to_string(),
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
            json_canon::to_string(&MediationError::InvalidMediationRequestFormat.json().0).unwrap()
        )
    }

    //------------------------------------------------------------------------
    // Helpers ---------------------------------------------------------------
    //------------------------------------------------------------------------

    pub fn _mediator_did(state: &AppState) -> String {
        state.diddoc.id.clone()
    }

    pub fn _edge_did() -> String {
        "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
    }

    pub fn _edge_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU",
                "d": "oItI6Jx-anGyhiDJIXtVAhzugOha05s-7_a5_CTs_V4"
            }"#,
        )
        .unwrap();

        LocalSecretsResolver::new(secret_id, &secret)
    }

    pub async fn _edge_pack_message(
        state: &AppState,
        msg: &Message,
        from: Option<String>,
        to: String,
    ) -> Result<String, DidcommError> {
        let (packed, _) = msg
            .pack_encrypted(
                &to,
                from.as_deref(),
                None,
                &state.did_resolver,
                &_edge_secrets_resolver(),
                &PackEncryptedOptions::default(),
            )
            .await?;

        Ok(packed)
    }

    pub async fn _edge_unpack_message(
        state: &AppState,
        msg: &str,
    ) -> Result<Message, DidcommError> {
        let (unpacked, _) = Message::unpack(
            msg,
            &state.did_resolver,
            &_edge_secrets_resolver(),
            &UnpackOptions::default(),
        )
        .await
        .expect("Unable to unpack");

        Ok(unpacked)
    }

    pub fn _inspect_dic_tags(vdic: &[CompactDIC]) -> Vec<&'static str> {
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
