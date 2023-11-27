use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use didcomm::{Message, UnpackOptions};

use super::error::MediationError;
use crate::{
    constant::{
        DIDCOMM_ENCRYPTED_MIME_TYPE, DIDCOMM_ENCRYPTED_SHORT_MIME_TYPE, MEDIATE_REQUEST_1_0,
        MEDIATE_REQUEST_2_0,
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
    if ![MEDIATE_REQUEST_1_0, MEDIATE_REQUEST_2_0].contains(&message.type_.as_str()) {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType.json(),
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
    mediation_request: &MediationRequest,
    message_type: &str,
) -> Result<(), Response> {
    if mediation_request.message_type != message_type {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType.json(),
        );

        return Err(response.into_response());
    }

    Ok(())
}
