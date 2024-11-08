use axum::response::{IntoResponse, Response};
use didcomm::Message;
use serde_json::json;
use uuid::Uuid;

use super::{
    constant::DISCOVER_FEATURE,
    errors::DiscoveryError,
    model::{Disclosures, DisclosuresContent},
};

// handle discover feature request
// takes a message and a vector of supported messaging protocols as PIURI
// https://didcomm.org/discover-features/2.0/
pub(crate) fn handle_query_request(
    message: Message,
    supported_protocols: Vec<String>,
) -> Result<Message, Response> {
    let mut disclosed_protocols: Vec<String> = Vec::new();

    let body = message.body.get("queries").and_then(|val| val.as_array());

    if body.is_none() {
        return Err(DiscoveryError::QueryNotFound.json().into_response());
    }

    let queries = body.unwrap();

    for value in queries.iter() {
        match value.get("feature-type") {
            Some(val) => {
                let val = val.as_str().unwrap();
                if val == "protocol" {
                    match value.get("match") {
                        Some(id) => {
                            if supported_protocols
                                .iter()
                                .find(|&id| id == &id.to_string())
                                .is_some()
                            {
                                disclosed_protocols.push(id.to_owned().to_string());
                            }
                        }
                        None => return Err(DiscoveryError::MalformedBody.json().into_response()),
                    }

                // Only support features of type protocol
                } else {
                    // do nothing
                }
            }
            None => return Err(DiscoveryError::MalformedBody.json().into_response()),
        };
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

    Ok(msg)
}
#[cfg(test)]
mod test {

    use didcomm::Message;
    use serde_json::json;
    use uuid::Uuid;

    use crate::discover_feature::{constant::QUERY_FEATURE, model::Queries};

    use super::handle_query_request;
    const TRUST: &str = "https://didcomm.org/trust-ping/2.0/ping";
    #[tokio::test]
    async fn test_get_supported_protocols() {
        let queries = json!({"feature-type": "protocol", "match": TRUST});

        let supported_protocols = vec![TRUST.to_string()];
        let body = Queries {
            queries: vec![queries],
        };
        let id = Uuid::new_v4().urn().to_string();

        let message = Message::build(id, QUERY_FEATURE.to_string(), json!(body)).finalize();
        match handle_query_request(message, supported_protocols) {
            Ok(result) => {
                assert!(result.body.get("disclosures").is_some());
                assert!(result.body.get("disclosures").unwrap().is_array());
                assert!(
                    result
                        .body
                        .get("disclosures")
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .len()
                        == 1
                );
                let content = result.body.get("disclosures").unwrap().as_array().unwrap()[0]
                    .as_object()
                    .unwrap()
                    .get("id")
                    .unwrap()
                    .as_str()
                    .unwrap();
                let content: String = serde_json::from_str(content).unwrap();
                assert_eq!(content, TRUST.to_string());
            }
            Err(e) => {
                panic!("This should not occur {:?}", e)
            }
        }
    }
    #[tokio::test]
    async fn test_unsupported_feature_type() {
        let queries = json!({"feature-type": "goal-code", "match": "org.didcomm"});

        let supported_protocols = vec![TRUST.to_string()];
        let body = Queries {
            queries: vec![queries],
        };
        let id = Uuid::new_v4().urn().to_string();

        let message = Message::build(id, QUERY_FEATURE.to_string(), json!(body)).finalize();
        match handle_query_request(message, supported_protocols) {
            Ok(result) => {
                assert!(result
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
