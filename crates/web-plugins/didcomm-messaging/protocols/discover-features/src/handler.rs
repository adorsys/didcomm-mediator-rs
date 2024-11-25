use std::{collections::HashSet, sync::Arc};

use axum::response::{IntoResponse, Response};
use didcomm::Message;
use serde_json::json;
use shared::{constants::DISCOVER_FEATURE, state::AppState};
use uuid::Uuid;

use super::{
    errors::DiscoveryError,
    model::{Disclosures, DisclosuresContent},
};

// handle discover feature request
// https://didcomm.org/discover-features/2.0/
pub fn handle_query_request(
    message: Message,
    state: Arc<AppState>,
) -> Result<Option<Message>, Response> {
    let mut disclosed_protocols: HashSet<String> = HashSet::new();
    let supported: &Vec<String>;

    let queries = message.body.get("queries").and_then(|val| val.as_array());

    if let Some(protocols) = &state.supported_protocols {
        supported = protocols;
        if let Some(queries) = queries {
            for value in queries {
                match value.get("feature-type") {
                    Some(val) => {
                        let val = val.as_str().unwrap_or_default();
                        if val == "protocol" {
                            match value.get("match") {
                                Some(id) => {
                                    let id = id.as_str().unwrap_or_default();

                                    if !id
                                        .ends_with(".*")
                                        .then(|| {
                                            supported
                                                .into_iter()
                                                .find(|protocol| protocol.contains(&id.to_string()))
                                                .is_some()
                                        })
                                        .is_some()
                                    {
                                        disclosed_protocols.insert(id.to_owned());
                                    }
                                    // wildcard scenario
                                    else {
                                        let parts: Vec<&str> = id.split(".*").collect();
                                        // stores the full protocol obtained when we have a match with wildcard
                                        let mut container: String = Default::default();

                                        if let Some(id) = parts.get(0) {
                                            supported
                                                .into_iter()
                                                .find(|protocol| protocol.contains(&id.to_string()))
                                                .is_some_and(|protocol| {
                                                    container = protocol.to_string();
                                                    return true;
                                                })
                                                .then(|| {
                                                    let parts: Vec<&str> =
                                                        container.split(id).collect();
                                                    // getting the minor version supported by the mediator when we have a request with a wildcard as minor
                                                    let minor = parts[1]
                                                        .to_string()
                                                        .chars()
                                                        .nth(1)
                                                        .unwrap_or_default();
                                                    let id = format!("{id}.{minor}");
                                                    disclosed_protocols.insert(id.to_string());
                                                });
                                        }
                                    }
                                }

                                None => return Err(DiscoveryError::MalformedBody.into_response()),
                            }
                        } else {
                            return Err(DiscoveryError::FeatureNOTSupported.into_response());
                        }
                    }
                    None => return Err(DiscoveryError::MalformedBody.into_response()),
                }
            }

            // build response body
            let msg = build_response(disclosed_protocols);
            Ok(Some(msg))
        } else {
            return Err(DiscoveryError::QueryNotFound.into_response());
        }
    } else {
        let msg = build_response(disclosed_protocols);
        Ok(Some(msg))
    }
}
fn build_response(disclosed_protocols: HashSet<String>) -> Message {
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

    msg
}
#[cfg(test)]
mod test {

    use std::{sync::Arc, vec};

    use did_utils::didcore::Document;
    use didcomm::Message;
    use keystore::tests::MockKeyStore;
    use serde_json::json;
    use shared::{
        constants::QUERY_FEATURE,
        repository::tests::{MockConnectionRepository, MockMessagesRepository},
        state::{AppState, AppStateRepository},
    };
    use uuid::Uuid;

    use crate::model::Queries;

    use super::handle_query_request;
    const MEDIATION: &str = "https://didcomm.org/coordinate-mediation/2.0";

    pub fn setup() -> Arc<AppState> {
        let public_domain = String::from("http://alice-mediator.com");

        let diddoc = Document::default();

        let repository = AppStateRepository {
            connection_repository: Arc::new(MockConnectionRepository::from(vec![])),
            message_repository: Arc::new(MockMessagesRepository::from(vec![])),
            keystore: Arc::new(MockKeyStore::new(vec![])),
        };

        let state = Arc::new(AppState::from(
            public_domain,
            diddoc,
            Some(vec![
                "https://didcomm.org/coordinate-mediation/2.0/mediate-request".to_string(),
            ]),
            Some(repository),
        ));

        state
    }

    #[tokio::test]
    async fn test_get_supported_protocols() {
        let queries = json!({"feature-type": "protocol", "match": MEDIATION});

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
                assert_eq!(
                    result
                        .clone()
                        .unwrap()
                        .body
                        .get("disclosures")
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .len(),
                    1
                );
                let id = serde_json::from_str::<String>(
                    &result
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
                        .to_string(),
                )
                .unwrap();

                assert_eq!(id, MEDIATION.to_string());
            }
            Err(e) => {
                panic!("This should not occur {:?}", e)
            }
        }
    }
    #[tokio::test]
    async fn test_get_supported_protocols_with_wildcard() {
        let queries = json!({"feature-type": "protocol", "match": "https://didcomm.org/coordinate-mediation/2.*"});

        // test duplicates in queries
        let body = Queries {
            queries: vec![queries.clone(), queries],
        };
        let id = Uuid::new_v4().urn().to_string();

        let message = Message::build(id, QUERY_FEATURE.to_string(), json!(body)).finalize();
        let state = setup();
        match handle_query_request(message, state) {
            Ok(result) => {
                println!("{:#?}", result.clone().unwrap());
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
                let id = serde_json::from_str::<String>(
                    &result
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
                        .to_string(),
                )
                .unwrap();

                assert_eq!(id, MEDIATION.to_string());
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
            Ok(_) => {
                panic!("This should'nt occur");
            }
            Err(_) => {}
        }
    }
}