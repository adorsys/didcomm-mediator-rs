use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use didcomm::Message;
use serde_json::json;
use uuid::Uuid;

use crate::web::AppState;

use super::{
    constant::DISCOVER_FEATURE,
    errors::DiscoveryError,
    model::{Disclosures, DisclosuresContent},
};

// handle discover feature request
// https://didcomm.org/discover-features/2.0/
pub(crate) fn handle_query_request(
    message: Message,
    state: Arc<AppState>,
) -> Result<Option<Message>, Response> {
    let mut disclosed_protocols: Vec<String> = Vec::new();

    let body = message.body.get("queries").and_then(|val| val.as_array());

    if let Some(queries) = body {
        for value in queries {
            match value.get("feature-type") {
                Some(val) => {
                    let val = val.as_str().unwrap();
                    if val.to_string() == "protocol" {
                        match value.get("match") {
                            Some(id) => {
                                let id = id.to_string();
                                let parts: Vec<&str> = id.split("/").collect();
                                let (name, version) = (parts[3], parts[4]);
                                let query_protocol = format!("{}/{}", name, version);

                                if let Some(supported_protocols) =
                                    &state.clone().supported_protocols
                                {
                                    if supported_protocols
                                        .iter()
                                        .find(|&disclose_protocol| {
                                            disclose_protocol == &query_protocol
                                        })
                                        .is_some()
                                    {
                                        disclosed_protocols.push(id);
                                    }
                                }
                            }
                            None => {
                                return Err(DiscoveryError::MalformedBody.json().into_response())
                            }
                        }
                    }
                }
                None => return Err(DiscoveryError::MalformedBody.json().into_response()),
            }
        }

        // build response body
        let mut body = Disclosures::new();
        for id in disclosed_protocols.iter() {
            let content = DisclosuresContent {
                feature_type: "protocol".to_string(),
                id: id.to_owned(),
                roles: None,
            };
            let content = json!(content);

            body.disclosures.push(content);
        }

        let id = Uuid::new_v4().urn().to_string();
        let msg = Message::build(id, DISCOVER_FEATURE.to_string(), json!(body)).finalize();

        Ok(Some(msg))
    } else {
        return Err(DiscoveryError::QueryNotFound.json().into_response());
    }
}
#[cfg(test)]
mod test {

    use std::{sync::Arc, vec};

    use didcomm::Message;
    use serde_json::json;
    use uuid::Uuid;

    use crate::{
        discover_feature::{constant::QUERY_FEATURE, model::Queries},
        repository::stateful::tests::{
            MockConnectionRepository, MockMessagesRepository, MockSecretsRepository,
        },
        util::{self, MockFileSystem},
        web::{AppState, AppStateRepository},
    };

    use super::handle_query_request;
    const TRUST: &str = "https://didcomm.org/trust-ping/2.0/ping";
    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let mut mock_fs = MockFileSystem;
        let storage_dirpath = std::env::var("STORAGE_DIRPATH").unwrap_or_else(|_| "/".to_owned());
        let diddoc: did_utils::didcore::Document =
            util::read_diddoc(&mock_fs, &storage_dirpath).unwrap();
        let keystore = util::read_keystore(&mut mock_fs, "").unwrap();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            secret_repository: Arc::new(MockSecretsRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
        };

        let state = Arc::new(AppState::from(
            public_domain,
            diddoc,
            keystore,
            Some(vec!["trust-ping/2.0".to_string()]),
            Some(repository),
        ));

        state
    }

    #[tokio::test]
    async fn test_get_supported_protocols() {
        let queries = json!({"feature-type": "protocol", "match": TRUST});

        let body = Queries {
            queries: vec![queries],
        };
        let id = Uuid::new_v4().urn().to_string();

        let message = Message::build(id, QUERY_FEATURE.to_string(), json!(body)).finalize();
        let state = setup();
        match handle_query_request(message, state) {
            Ok(result) => {
                assert!(result.clone().unwrap().body.get("disclosures").is_some());
                assert!(result
                    .clone()
                    .unwrap()
                    .body
                    .get("disclosures")
                    .unwrap()
                    .is_array());
                assert!(
                    result
                        .clone()
                        .unwrap()
                        .body
                        .get("disclosures")
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .len()
                        == 1
                );

                assert_eq!(
                    serde_json::from_str::<String>(
                        result
                            .unwrap()
                            .body
                            .get("disclosures")
                            .unwrap()
                            .as_array()
                            .unwrap()[0]
                            .as_object()
                            .unwrap()
                            .get("id")
                            .unwrap()
                            .as_str()
                            .unwrap()
                    )
                    .unwrap(),
                    TRUST.to_string()
                );
            }
            Err(e) => {
                panic!("This should not occur {:?}", e)
            }
        }
    }
    #[tokio::test]
    async fn test_unsupported_feature_type() {
        let queries = json!({"feature-type": "goal-code", "match": "org.didcomm"});
        let state = setup();
        let body = Queries {
            queries: vec![queries],
        };
        let id = Uuid::new_v4().urn().to_string();

        let message = Message::build(id, QUERY_FEATURE.to_string(), json!(body)).finalize();
        match handle_query_request(message, state) {
            Ok(result) => {
                assert!(result
                    .unwrap()
                    .body
                    .get("disclosures")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .is_empty())
            }
            Err(e) => {
                panic!("This should not occur: {:#?}", e)
            }
        }
    }
}
