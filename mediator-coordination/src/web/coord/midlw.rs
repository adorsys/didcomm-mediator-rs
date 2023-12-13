use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use didcomm::{error::ErrorKind as DidcommErrorKind, Message, PackEncryptedOptions, UnpackOptions};
use serde_json::{json, Value};

use super::error::MediationError;
use crate::{
    constant::{
        DIDCOMM_ENCRYPTED_MIME_TYPE, DIDCOMM_ENCRYPTED_SHORT_MIME_TYPE, MEDIATE_REQUEST_2_0,
        MEDIATE_REQUEST_DIC_1_0,
    },
    didcomm::bridge::{LocalDIDResolver, LocalSecretsResolver},
    model::coord::MediationRequest,
};

macro_rules! run {
    ($expression:expr) => {
        match $expression {
            Ok(res) => res,
            Err(err) => return err,
        }
    };
}
pub(crate) use run;

/// Ensure header content-type match `application/didcomm-encrypted+json` or `didcomm-encrypted+json`
pub fn ensure_content_type_is_didcomm_encrypted(headers: &HeaderMap) -> Result<(), Response> {
    let content_type = headers.get(CONTENT_TYPE);

    if content_type.is_none()
        || [
            DIDCOMM_ENCRYPTED_MIME_TYPE,
            DIDCOMM_ENCRYPTED_SHORT_MIME_TYPE,
        ]
        .iter()
        .all(|e| e != content_type.unwrap())
    {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::NotDidcommEncryptedPayload.json(),
        );

        return Err(response.into_response());
    }

    Ok(())
}

/// Decrypt assumed authcrypt'd didcomm messaged
pub async fn unpack_request_message(
    msg: &str,
    did_resolver: &LocalDIDResolver,
    secrets_resolver: &LocalSecretsResolver,
) -> Result<Message, Response> {
    let res = Message::unpack(
        msg,
        did_resolver,
        secrets_resolver,
        &UnpackOptions::default(),
    )
    .await;

    let (plain_message, metadata) = res.map_err(|_| {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::MessageUnpackingFailure.json(),
        );

        response.into_response()
    })?;

    if !metadata.encrypted {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::MalformedDidcommEncrypted.json(),
        );

        return Err(response.into_response());
    }

    if plain_message.from.is_none() || !metadata.authenticated || metadata.anonymous_sender {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::AnonymousPacker.json(),
        );

        return Err(response.into_response());
    }

    Ok(plain_message)
}

/// Validate that JWM's indicative body type is a mediation request
pub fn ensure_jwm_type_is_mediation_request(message: &Message) -> Result<(), Response> {
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
    let transport_decoration = message
        .extra_headers
        .get("~transport")
        .unwrap_or(&Value::Null);

    if !transport_decoration.is_object()
        || transport_decoration
            .as_object()
            .unwrap()
            .get("return_route")
            != Some(&json!("all"))
    {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::NoReturnRouteAllDecoration.json(),
        );

        return Err(response.into_response());
    }

    Ok(())
}

/// Parse message body into mediation request
pub fn parse_message_body_into_mediation_request(
    message: &Message,
) -> Result<MediationRequest, Response> {
    serde_json::from_value::<MediationRequest>(message.body.clone()).map_err(|_| {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMediationRequestFormat.json(),
        );

        response.into_response()
    })
}

/// Validate that mediation request's URI type is as expected
pub fn ensure_mediation_request_type(
    mediation_request: &Value,
    message_type: &str,
) -> Result<(), Response> {
    if mediation_request.get("@type") != Some(&json!(message_type)) {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType.json(),
        );

        return Err(response.into_response());
    }

    Ok(())
}

/// Pack response message
pub async fn pack_response_message(
    msg: &Message,
    did_resolver: &LocalDIDResolver,
    secrets_resolver: &LocalSecretsResolver,
) -> Result<Value, Response> {
    let from = msg.from.as_ref();
    let to = msg.to.as_ref().and_then(|v| v.get(0));

    if from.is_none() || to.is_none() {
        let response = (
            StatusCode::INTERNAL_SERVER_ERROR,
            MediationError::MessagePackingFailure(DidcommErrorKind::Malformed).json(),
        );

        return Err(response.into_response());
    }

    msg.pack_encrypted(
        to.unwrap(),
        from.map(|x| x.as_str()),
        None,
        did_resolver,
        secrets_resolver,
        &PackEncryptedOptions::default(),
    )
    .await
    .map(|(packed_message, _metadata)| serde_json::from_str(&packed_message).unwrap())
    .map_err(|err| {
        let response = (
            StatusCode::INTERNAL_SERVER_ERROR,
            MediationError::MessagePackingFailure(err.kind()).json(),
        );

        response.into_response()
    })
}

#[cfg(test)]
mod tests {
    use super::{super::handler::tests::*, *};

    #[cfg(feature = "stateless")]
    use crate::model::stateless::coord::{
        MediationRequest as StatelessMediationRequest, MediatorService,
    };

    #[tokio::test]
    async fn test_ensure_content_type_is_didcomm_encrypted() {
        /* Positive cases */

        let headers: HeaderMap = [(CONTENT_TYPE, DIDCOMM_ENCRYPTED_MIME_TYPE.parse().unwrap())]
            .into_iter()
            .collect();
        assert!(ensure_content_type_is_didcomm_encrypted(&headers).is_ok());

        let headers: HeaderMap = [(
            CONTENT_TYPE,
            DIDCOMM_ENCRYPTED_SHORT_MIME_TYPE.parse().unwrap(),
        )]
        .into_iter()
        .collect();
        assert!(ensure_content_type_is_didcomm_encrypted(&headers).is_ok());

        /* Negative cases */

        let headers: HeaderMap = [].into_iter().collect();
        _assert_midlw_err(
            ensure_content_type_is_didcomm_encrypted(&headers).unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::NotDidcommEncryptedPayload,
        )
        .await;

        let headers: HeaderMap = [(CONTENT_TYPE, "application/json".parse().unwrap())]
            .into_iter()
            .collect();
        _assert_midlw_err(
            ensure_content_type_is_didcomm_encrypted(&headers).unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::NotDidcommEncryptedPayload,
        )
        .await;
    }

    #[tokio::test]
    async fn test_unpack_message_works() {
        let (_, state) = setup();

        let plain_msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!({
                "@id": "id_alice_mediation_request",
                "@type": MEDIATE_REQUEST_DIC_1_0,
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        let packed_msg =
            _edge_pack_message(&state, &plain_msg, Some(_edge_did()), _mediator_did(&state))
                .await
                .unwrap();

        let unpacked_msg =
            unpack_request_message(&packed_msg, &state.did_resolver, &state.secrets_resolver)
                .await
                .unwrap();
        assert_eq!(unpacked_msg, plain_msg);
    }

    #[tokio::test]
    async fn test_unpack_non_destinated_message() {
        let (_, state) = setup();

        let plain_msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!({
                "@id": "id_alice_mediation_request",
                "@type": MEDIATE_REQUEST_DIC_1_0,
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_edge_did())
        .from(_edge_did())
        .finalize();

        let packed_msg = _edge_pack_message(&state, &plain_msg, Some(_edge_did()), _edge_did())
            .await
            .unwrap();

        _assert_midlw_err(
            unpack_request_message(&packed_msg, &state.did_resolver, &state.secrets_resolver)
                .await
                .unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::MessageUnpackingFailure,
        )
        .await;
    }

    #[tokio::test]
    async fn test_unpack_non_encrypted_message() {
        let (_, state) = setup();

        let plain_msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!({
                "@id": "id_alice_mediation_request",
                "@type": MEDIATE_REQUEST_DIC_1_0,
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_edge_did())
        .from(_edge_did())
        .finalize();

        let msg = plain_msg.pack_plaintext(&state.did_resolver).await.unwrap();

        _assert_midlw_err(
            unpack_request_message(&msg, &state.did_resolver, &state.secrets_resolver)
                .await
                .unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::MalformedDidcommEncrypted,
        )
        .await;

        let (msg, _) = plain_msg
            .pack_signed(
                &_edge_did(),
                &state.did_resolver,
                &_edge_signing_secrets_resolver(),
            )
            .await
            .unwrap();

        _assert_midlw_err(
            unpack_request_message(&msg, &state.did_resolver, &state.secrets_resolver)
                .await
                .unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::MalformedDidcommEncrypted,
        )
        .await;
    }

    #[tokio::test]
    async fn test_unpack_anonymously_encrypted_message() {
        let (_, state) = setup();

        let plain_msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!({
                "@id": "id_alice_mediation_request",
                "@type": MEDIATE_REQUEST_DIC_1_0,
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        // No sender
        let packed_msg = _edge_pack_message(&state, &plain_msg, None, _mediator_did(&state))
            .await
            .unwrap();

        _assert_midlw_err(
            unpack_request_message(&packed_msg, &state.did_resolver, &state.secrets_resolver)
                .await
                .unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::AnonymousPacker,
        )
        .await;
    }

    #[tokio::test]
    async fn test_unpack_anonymously_encrypted_message_but_signed() {
        let (_, state) = setup();

        let plain_msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_DIC_1_0.to_string(),
            json!({
                "@id": "id_alice_mediation_request",
                "@type": MEDIATE_REQUEST_DIC_1_0,
                "did": "did:key:alice_identity_pub@alice_mediator",
                "services": ["inbox", "outbox"]
            }),
        )
        .to(_mediator_did(&state))
        .from(_edge_did())
        .finalize();

        // No sender but signed
        let (packed_msg, _) = plain_msg
            .pack_encrypted(
                &_mediator_did(&state),
                None,
                Some(&_edge_did()), // sign_by
                &state.did_resolver,
                &_edge_signing_secrets_resolver(),
                &PackEncryptedOptions::default(),
            )
            .await
            .unwrap();

        _assert_midlw_err(
            unpack_request_message(&packed_msg, &state.did_resolver, &state.secrets_resolver)
                .await
                .unwrap_err(),
            StatusCode::BAD_REQUEST,
            MediationError::AnonymousPacker,
        )
        .await;
    }

    #[tokio::test]
    async fn test_ensure_jwm_type_is_mediation_request() {
        let (_, state) = setup();

        /* Positive cases */

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            MEDIATE_REQUEST_2_0.to_string(),
            json!({
                "@id": "id_alice_mediation_request",
                "@type": MEDIATE_REQUEST_2_0,
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
                "@id": "id_alice_mediation_request",
                "@type": MEDIATE_REQUEST_DIC_1_0,
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
                "@id": "id_alice_mediation_request",
                "@type": MEDIATE_REQUEST_2_0,
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
        let (_, state) = setup();

        macro_rules! unfinalized_msg {
            () => {
                Message::build(
                    "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
                    MEDIATE_REQUEST_2_0.to_string(),
                    json!({
                        "@id": "id_alice_mediation_request",
                        "@type": MEDIATE_REQUEST_2_0,
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
            .header(
                "~transport".into(),
                json!({
                    "return_route": "all"
                }),
            )
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
            .header(
                "~transport".into(),
                json!({
                    "return_route": "none"
                }),
            )
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
        let MediationRequest::Stateless(parsed_mediation_request) = parsed_mediation_request else { panic!() };
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
            MediationError::InvalidMediationRequestFormat,
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

    #[tokio::test]
    async fn test_pack_response_message_works() {
        let (_, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            "application/json".to_string(),
            json!({
                "content": "a quick brown fox jumps over the lazy dog"
            }),
        )
        .to(_edge_did())
        .from(_mediator_did(&state))
        .finalize();

        let packed = pack_response_message(&msg, &state.did_resolver, &state.secrets_resolver)
            .await
            .unwrap();
        let packed_str = json_canon::to_string(&packed).unwrap();
        assert!(_edge_unpack_message(&state, &packed_str).await.is_ok());
    }

    #[tokio::test]
    async fn test_pack_response_message_fails_on_any_end_missing() {
        let (_, state) = setup();

        macro_rules! unfinalized_msg {
            () => {
                Message::build(
                    "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
                    "application/json".to_string(),
                    json!({
                        "content": "a quick brown fox jumps over the lazy dog"
                    }),
                )
            };
        }

        let msgs = [
            unfinalized_msg!().to(_edge_did()).finalize(),
            unfinalized_msg!().from(_mediator_did(&state)).finalize(),
            unfinalized_msg!().finalize(),
        ];

        for msg in msgs {
            _assert_midlw_err(
                pack_response_message(&msg, &state.did_resolver, &state.secrets_resolver)
                    .await
                    .unwrap_err(),
                StatusCode::INTERNAL_SERVER_ERROR,
                MediationError::MessagePackingFailure(DidcommErrorKind::Malformed),
            )
            .await;
        }
    }

    #[tokio::test]
    async fn test_pack_response_message_on_unsupported_receiving_did() {
        let (_, state) = setup();

        let msg = Message::build(
            "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
            "application/json".to_string(),
            json!({
                "content": "a quick brown fox jumps over the lazy dog"
            }),
        )
        .to(String::from("did:sov:WRfXPg8dantKVubE3HX8pw"))
        .from(_mediator_did(&state))
        .finalize();

        _assert_midlw_err(
            pack_response_message(&msg, &state.did_resolver, &state.secrets_resolver)
                .await
                .unwrap_err(),
            StatusCode::INTERNAL_SERVER_ERROR,
            MediationError::MessagePackingFailure(DidcommErrorKind::Unsupported),
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
