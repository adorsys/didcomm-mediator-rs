use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use didcomm::Message;
use hyper::StatusCode;
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
pub(crate) fn handle_query_request(message: Message) -> Result<Option<Message>, Response> {
    let mut disclosed_protocols: Vec<String> = Vec::new();
    let supported_protocols: Vec<String> = Vec::new();

    let body = message.body.get("queries");

    if body.is_none() {
        return Err(StatusCode::BAD_REQUEST.into_response());
    }

    let querie = body.unwrap().as_array();
    if querie.is_none() {
        return Err(DiscoveryError::MalformedBody.json().into_response());
    }
    let queries = querie.unwrap();
    let _ = queries.iter().map(|value| {
        Ok(match value.get("feature-type") {
            Some(protocol) => {
                if protocol.to_string() == "protocol" {
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
                }
            }
            None => return Err(DiscoveryError::MalformedBody.json().into_response()),
        })
    });
    // build response body
    let _ = disclosed_protocols.iter().map(|id| {
        let mut body = Disclosures::new();
        let content = DisclosuresContent {
            feature_type: "feature_type".to_string(),
            id: id.to_string(),
            roles: None,
        };
        body.disclosures.push(content);
    });
    let id = Uuid::new_v4().urn().to_string();

    Ok(Some(
        Message::build(id, DISCOVER_FEATURE.to_string(), json!(body)).finalize(),
    ))
}
#[cfg(test)]
mod test {
    use axum::body::HttpBody;
    use didcomm::Message;
    use serde_json::json;
    use uuid::Uuid;

    use crate::discover_feature::model::Queries;

    use super::{handle_query_request, DISCOVER_FEATURE};

  
  #[tokio::test]
  async fn test_get_queries() {
    let queries = json!({"feature-type": "https://didcomm.org/discover-features/2.0/disclose"});
    let body = Queries { queries: vec![queries] };
    let id = Uuid::new_v4().urn().to_string();

    let message = Message::build(id, DISCOVER_FEATURE.to_string(), json!(body)).finalize();
     match handle_query_request(message) {
      Ok(result) => {
        let msg = result.unwrap();
        println!("{:#?}", msg);
      },
      Err(e) => {
     
        println!("{:?}", e);
      }
     }
  }
}