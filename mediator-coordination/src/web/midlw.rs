use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use didcomm::{Message, UnpackOptions};
use std::sync::Arc;

use super::{error::MediationError, AppState};
use crate::{
    constant::{DIDCOMM_ENCRYPTED_MIME_TYPE, DIDCOMM_ENCRYPTED_SHORT_MIME_TYPE},
    didcomm::bridge::{LocalDIDResolver, LocalSecretsResolver},
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
