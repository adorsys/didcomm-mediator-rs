mod stateful;
#[cfg(feature = "stateless")]
mod stateless;

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
        #[cfg(feature = "stateless")]
        MediationRequest::Stateless(req) => midlw::run!(
            stateless::process_plain_mediation_request_over_dics(&state, &plain_message, &req)
                .await
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
pub mod tests {
    use super::*;

    use axum::Router;
    use did_utils::key_jwk::jwk::Jwk;
    use didcomm::{
        error::Error as DidcommError, secrets::SecretsResolver, Message, PackEncryptedOptions,
        UnpackOptions,
    };

    use crate::{
        didcomm::bridge::LocalSecretsResolver,
        util::{self, MockFileSystem},
        web,
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

    //------------------------------------------------------------------------
    // Helpers ---------------------------------------------------------------
    //------------------------------------------------------------------------

    pub fn _mediator_did(state: &AppState) -> String {
        state.diddoc.id.clone()
    }

    pub fn _edge_did() -> String {
        "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
    }

    pub fn _edge_signing_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "d": "UXBdR4u4bnHHEaDK-dqE04DIMvegx9_ZOjm--eGqHiI",
                "x": "Fpf4juyZWYUNmC8Bv87MmFLDWApxqOYYZUhWyiD7lSo"
            }"#,
        )
        .unwrap();

        LocalSecretsResolver::new(secret_id, &secret)
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
}
