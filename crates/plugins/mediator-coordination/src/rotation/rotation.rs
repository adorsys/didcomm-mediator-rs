use super::errors::RotationError;
use crate::{
    didcomm::bridge::LocalDIDResolver,
    jose::jws::{self, verify_compact_jws},
    model::stateful::entity::Connection,
};
use axum::response::{IntoResponse, Response};
use base64::{decode_config, URL_SAFE_NO_PAD};
use database::Repository;
use didcomm::{did::DIDResolver, FromPrior, Message};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use hmac::{Hmac, Mac};
use hyper::StatusCode;
use jsonwebtoken::{
    crypto::verify,
    jwk::{self, Jwk, KeyAlgorithm},
    Algorithm, DecodingKey,
};
use jwt::VerifyWithKey;
use mongodb::bson::doc;
use serde_json::Error;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::error::Error as err;
use std::sync::Arc;
pub enum Errors {
    Error0(RotationError),
    Error1(Error),
    Error2(Response),
}

pub async fn did_rotation(
    msg: Message,
    conection_repos: &Arc<dyn Repository<Connection>>,
) -> Result<(), Errors> {

    // Check if from_prior is not none
    if msg.from_prior.is_some() {

        let jwt = msg.from_prior.unwrap();
        let did_resolver = LocalDIDResolver::default();
        let (from_prior, kid) = FromPrior::unpack(&jwt, &did_resolver)
            .await
            .map_err(|_| Errors::Error2(RotationError::InvalidFromPrior.json().into_response()))?;

        let prev = from_prior.iss;

        // validate if did is  known
        let _connection = match conection_repos
            .find_one_by(doc! {"client_did": &prev})
            .await
            .unwrap()
        {
            Some(connection) => {
                let (signature, message) = get_jwt_signature_payload(&jwt).unwrap();
                let key = jsonwebtoken::DecodingKey::from_secret(kid.as_bytes());

                // validate jwt signatures with previous did kid
                if verify(signature, message.as_bytes(), &key, Algorithm::EdDSA).unwrap() {
                    
                    // stored the new did for communication
                    let new = from_prior.sub;
                    connection.client_did.replace(&prev, &new);
                } else {
                    let response = (
                        StatusCode::UNAUTHORIZED,
                        RotationError::InvalidSignature.json(),
                    );
                    return Err(Errors::Error2(response.into_response()))?;
                };
            }
            None => {
                return Err(Errors::Error0(RotationError::RotationError))?;
            }
        };
    }
    Ok(())
}
fn get_jwt_signature_payload(jwt: &str) -> Result<(&str, &str), Box<dyn err>> {
    // Split the JWT into its three parts (header, payload, and signature)
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".into());
    }
    let message = parts[1];
    let signature = parts[2];
    Ok((signature, message))
}

#[cfg(test)]
mod test {

    pub fn prev_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkeWXQx7Ycpuj4PhXB1GHRinwozrkjn4yot6a3PCU3citF#z6MkeWXQx7Ycpuj4PhXB1GHRinwozrkjn4yot6a3PCU3citF";
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "ANYekDNsggaD4B3ilknnvaPOheJj7jfqNAq7Powb75g",
                "d": "ataQeHO0ATp7DJmr2L7WQ0PF1vjnHKvsn0zkaUNCVjg"
            }"#,
        )
        .unwrap();

        LocalSecretsResolver::new(&secret_id, &secret)
    }
    pub fn new_secrets_resolver() -> impl SecretsResolver {
        let secret_id = "did:key:z6MkwKfDFAK49Lb9D6HchFiCXdcurRUSFrbnwDBk5qFZeHA3#z6MkwKfDFAK49Lb9D6HchFiCXdcurRUSFrbnwDBk5qFZeHA3".to_owned();
        let secret: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "X25519",
                "x": "ZlJzHqy2dLrDQNlV15O3zDOIXpWVQnq6VtiVZ78O0hY",
                "d": "8OK7-1IVMdcM86PZzYKsbIi3kCJ-RxI8XFKe9JEcF2Y"
            }"#,
        )
        .unwrap();

        LocalSecretsResolver::new(&secret_id, &secret)
    }
    pub fn prev_did() -> String {
        "did:key:z6MkeWXQx7Ycpuj4PhXB1GHRinwozrkjn4yot6a3PCU3citF".to_string()
    }
    pub fn new_did() -> String {
        "did:key:z6MkwKfDFAK49Lb9D6HchFiCXdcurRUSFrbnwDBk5qFZeHA3".to_string()
    }
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let storage_dirpath: String =
            std::env::var("STORAGE_DIRPATH").unwrap_or_else(|_| "/".to_owned());
        let diddoc: did_utils::didcore::Document =
            util::read_diddoc(&mock_fs, &storage_dirpath).unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(_initial_connections())),
            secret_repository: Arc::new(MockSecretsRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
        };

        let state = Arc::new(AppState::from(
            public_domain,
            diddoc,
            keystore,
            Some(repository),
        ));

        state
    }
    fn _initial_connections() -> Vec<Connection> {
        let _recipient_did = prev_did();

        let connections = format!(
            r##"[
                {{
                "_id": {{
                    "$oid": "6580701fd2d92bb3cd291b2a"
                    }},
                    
                    "client_did": "{_recipient_did}",
                    "mediator_did": "did:web:alice-mediator.com:alice_mediator_pub",
                    "routing_did": "did:key:generated",
                    "keylist": [
                        "{_recipient_did}"
                        ]
                    }}
                    ]"##
        );

        serde_json::from_str(&connections).unwrap()
    }
    use std::sync::Arc;

    use did_utils::jwk::Jwk;
    use didcomm::{secrets::SecretsResolver, FromPrior, Message, PackEncryptedOptions, UnpackOptions};
    use serde_json::json;
    use uuid::Uuid;

    use crate::{
        didcomm::bridge::{LocalDIDResolver, LocalSecretsResolver},
        model::stateful::entity::Connection,
        repository::stateful::tests::{
            MockConnectionRepository, MockMessagesRepository, MockSecretsRepository,
        },
        rotation::rotation::did_rotation,
        util::{self, MockFileSystem},
        web::{AppState, AppStateRepository},
    };

    #[tokio::test]
    async fn test_did_rotation() {
        let state = &setup();

        let from_prior = FromPrior {
            iss: prev_did(),
            sub: new_did(),
            aud: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };
        // let claims = serde_json::to_string(&from_prior).unwrap();
        let did_resolver = LocalDIDResolver::default();
        let kid = "did:key:z6MkeWXQx7Ycpuj4PhXB1GHRinwozrkjn4yot6a3PCU3citF#z6MkeWXQx7Ycpuj4PhXB1GHRinwozrkjn4yot6a3PCU3citF";
        let (jwt, _kid) = from_prior
            .pack(Some(&kid), &did_resolver, &prev_secrets_resolver())
            .await
            .unwrap();
        println!("{jwt}");

        let msg = Message::build(
            Uuid::new_v4().to_string(),
            "example/v1".to_owned(),
            json!(""),
        )
        .to("did:web:alice-mediator.com:alice_mediator_pub".to_string())
        .from(new_did())
        .from_prior(jwt)
        .finalize();
        let (msg, _) = msg
            .pack_encrypted(
                "did:web:alice-mediator.com:alice_mediator_pub",
                Some(&new_did()),
                None,
                &state.did_resolver,
                &state.secrets_resolver, // should be new_did_secrets
                &PackEncryptedOptions::default(),
            )
            .await
            .unwrap();

        // Mediator in action
        let did_resolver = LocalDIDResolver::default();
        let secrets_resolver = prev_secrets_resolver();

        let msg = Message::unpack(
            &msg,
            &did_resolver,
            &secrets_resolver,
            &UnpackOptions::default(),
        )
        .await
        .unwrap();
        let AppStateRepository {
            connection_repository,
            ..
        } = state.repository.as_ref().unwrap();
        let _ = did_rotation(msg.0, connection_repository).await;
    }
}
