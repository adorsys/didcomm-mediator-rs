use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    sync::Arc,
};

use axum::{
    http::version,
    response::{IntoResponse, Response},
};
use didcomm::Message;
use serde::de::value;
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
    let mut disclosed_protocols: Vec<String> = Vec::new();

    let queries = message.body.get("queries").and_then(|val| val.as_array());

    let mut extracted_protocol = HashSet::new();
    let mut wildcard_protocol: HashSet<String> = HashSet::new();

    if let Some(supported_protocol) = &state.supported_protocols {
        for protocol in supported_protocol {
            let parts: Vec<&str> = protocol.split("/").collect();
            let (name, version) = (parts[3], parts[4]);
            let protocol = format!("{}/{}", name, version);
            extracted_protocol.insert(protocol);
        }
        // wildcard scenario
        for protocol in extracted_protocol.clone() {
            let value: Vec<&str> = protocol.split(".").collect();
            wildcard_protocol.insert(value[0].to_string());
        }
    };

    if let Some(queries) = queries {
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
                                let version: Vec<&str> = version.split("\"").collect();
                                let version = version[0];

                                let semver: Vec<&str> = version.split(".").collect();
                                let minor: &str = semver[1];
                                if minor != "*" {
                                    let protocol = format!("{}/{}", name, version);

                                    if let Some(_) = extracted_protocol.get(&protocol) {
                                        disclosed_protocols.push(id.clone());
                                    }
                                }
                                if minor == "*" {
                                    let major = semver[0];
                                    let protocol = format!("{}/{}", name, major);
                                    if let Some(_) = wildcard_protocol.get(&protocol) {
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

        println!("{:#?}", msg);
        Ok(Some(msg))
    } else {
        return Err(DiscoveryError::QueryNotFound.json().into_response());
    }
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
    const MEDIATION: &str = "https://didcomm.org/coordinate-mediation/2.*";
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
                    MEDIATION.to_string()
                );
            }
            Err(e) => {
                panic!("This should not occur {:?}", e)
            }
        }
    }
    #[tokio::test]
    async fn test_get_wildcard_supported_protocols() {
        let queries = json!({"feature-type": "protocol", "match": "https://didcomm.org/coordinate-mediation/2.*"});

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
                    MEDIATION.to_string()
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
