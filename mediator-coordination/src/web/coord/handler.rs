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

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::Body,
        http::{Method, Request},
        Router,
    };
    use did_utils::key_jwk::jwk::Jwk;
    use didcomm::{
        error::Error as DidcommError, secrets::SecretsResolver, PackEncryptedOptions, UnpackOptions,
    };
    use serde_json::Value;
    use tower::util::ServiceExt;

    use crate::{
        didcomm::bridge::LocalSecretsResolver,
        jose::jws,
        util::{self, MockFileSystem},
        web,
    };

    fn setup() -> (Router, AppState) {
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
    async fn test_comprehensive_mediate_grant_response() {
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

        assert_eq!(msg.type_, MEDIATE_GRANT_2_0);
        assert_eq!(msg.from.unwrap(), _mediator_did(&state));
        assert_eq!(msg.to.unwrap(), [_edge_did()]);

        let mediate_grant: MediationGrant = serde_json::from_value(msg.body).unwrap();

        assert!(mediate_grant.id.starts_with("urn:uuid:"));
        assert_eq!(mediate_grant.message_type, MEDIATE_GRANT_2_0);
        assert_eq!(mediate_grant.endpoint, state.public_domain);

        assert_eq!(mediate_grant.dic.len(), 2);
        assert!({
            let mut iter = mediate_grant.dic.iter();
            iter.any(|dic| matches!(dic, CompactDIC::Inbox(_)))
        });
        assert!({
            let mut iter = mediate_grant.dic.iter();
            iter.any(|dic| matches!(dic, CompactDIC::Outbox(_)))
        });
        assert!({
            let mut iter = mediate_grant.dic.iter();
            iter.all(|dic| {
                jws::verify_compact_jws(&dic.plain_jws(), &state.assertion_jwk.1).is_ok()
            })
        });
    }

    //------------------------------------------------------------------------
    // Helpers ---------------------------------------------------------------
    //------------------------------------------------------------------------

    fn _mediator_did(state: &AppState) -> String {
        state.diddoc.id.clone()
    }

    fn _edge_did() -> String {
        "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
    }

    fn _edge_secrets_resolver() -> impl SecretsResolver {
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

    async fn _edge_pack_message(
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

    async fn _edge_unpack_message(state: &AppState, msg: &str) -> Result<Message, DidcommError> {
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
}
