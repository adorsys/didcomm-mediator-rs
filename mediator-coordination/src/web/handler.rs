use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};
use didcomm::Message;
use hyper::{header::CONTENT_TYPE, StatusCode};
use std::sync::Arc;

use crate::{
    constant::{DIDCOMM_ENCRYPTED_MIME_TYPE, KEYLIST_UPDATE_2_0},
    web::{self, error::MediationError, AppState},
};

#[axum::debug_handler]
pub async fn process_didcomm_message(
    State(state): State<Arc<AppState>>,
    Extension(message): Extension<Message>,
) -> Response {
    let msg = serde_json::to_string_pretty(&message).unwrap();
    tracing::info!("request: {msg}");

    let delegate_response = match message.type_.as_str() {
        KEYLIST_UPDATE_2_0 => {
            web::coord::handler::stateful::process_plain_keylist_update_message(
                Arc::clone(&state),
                message,
            )
            .await
        }
        _ => {
            let response = (
                StatusCode::BAD_REQUEST,
                MediationError::UnsupportedOperation.json(),
            );

            return response.into_response();
        }
    };

    process_response_from_delegate_handler(state, delegate_response).await
}

async fn process_response_from_delegate_handler(
    state: Arc<AppState>,
    response: Result<Message, Response>,
) -> Response {
    // Extract plain message or early return error response
    let plain_response_message = match response {
        Ok(message) => message,
        Err(response) => return response,
    };

    let msg = serde_json::to_string_pretty(&plain_response_message).unwrap();
    tracing::info!("response: {msg}");

    // Pack response message
    let packed_message = match web::midlw::pack_response_message(
        &plain_response_message,
        &state.did_resolver,
        &state.secrets_resolver,
    )
    .await
    {
        Ok(packed) => packed,
        Err(response) => return response,
    };

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
        repository::stateful::coord::tests::MockConnectionRepository,
        util::{self, MockFileSystem},
        web::{self, AppStateRepository},
    };

    pub fn setup() -> (Router, Arc<AppState>) {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let diddoc = util::read_diddoc(&mock_fs, "").unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
        };

        let state = Arc::new(AppState::from(
            public_domain,
            diddoc,
            keystore,
            Some(repository),
        ));
        let app = web::routes(Arc::clone(&state));

        (app, state)
    }

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
