use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use didcomm::{error::ErrorKind as DidcommErrorKind, Message, PackEncryptedOptions, UnpackOptions};
use serde_json::Value;
use std::sync::Arc;

use super::{error::MediationError, AppState, AppStateRepository};
use crate::{
    constant::{DIDCOMM_ENCRYPTED_MIME_TYPE, DIDCOMM_ENCRYPTED_SHORT_MIME_TYPE},
    didcomm::bridge::{LocalDIDResolver, LocalSecretsResolver},
    rotation::{errors::RotationError, rotation::did_rotation},
};

/// Middleware to unpack DIDComm messages for unified handler
pub async fn unpack_didcomm_message(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    // Enforce request content type to match `didcomm-encrypted+json`

    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|header| header.to_str().ok());

    if let Err(response) = content_type_is_didcomm_encrypted(content_type) {
        return response;
    }

    // Extract request payload

    let (parts, body) = request.into_parts();
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(_) => {
            let response = (
                StatusCode::BAD_REQUEST,
                MediationError::UnparseablePayload.json(),
            );

            return response.into_response();
        }
    };

    let payload = String::from_utf8_lossy(&bytes);

    // Attempt to unpack request payload

    let AppState {
        did_resolver,
        secrets_resolver,
        ..
    } = state.as_ref();

    match unpack_payload(&payload, did_resolver, secrets_resolver).await {
        Ok(plain_message) => {
            // check for and handle did rotation
            let AppStateRepository {
                connection_repository,
                ..
            } = state.repository.as_ref().unwrap();
            match did_rotation(plain_message.clone(), connection_repository).await {
                Ok(_) => {}
                Err(err) => {
                    let response = (StatusCode::BAD_REQUEST, err);
                    return response.into_response();
                }
            };

            let mut request = Request::from_parts(parts, Body::from(bytes));
            request.extensions_mut().insert(plain_message);

            next.run(request).await
        }
        Err(response) => response,
    }
}

/// Check if `content_type` match `application/didcomm-encrypted+json` or `didcomm-encrypted+json`
fn content_type_is_didcomm_encrypted(content_type: Option<&str>) -> Result<(), Response> {
    if content_type.is_none()
        || ![
            DIDCOMM_ENCRYPTED_MIME_TYPE,
            DIDCOMM_ENCRYPTED_SHORT_MIME_TYPE,
        ]
        .contains(&content_type.unwrap())
    {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::NotDidcommEncryptedPayload.json(),
        );

        return Err(response.into_response());
    }

    Ok(())
}

/// Decrypt assumed authcrypt'd didcomm message
async fn unpack_payload(
    payload: &str,
    did_resolver: &LocalDIDResolver,
    secrets_resolver: &LocalSecretsResolver,
) -> Result<Message, Response> {
    let res = Message::unpack(
        payload,
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
    use super::*;
    use crate::web::handler::tests::*;

    use serde_json::json;

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
