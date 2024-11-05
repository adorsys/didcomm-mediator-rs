use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use didcomm::Message;
use hyper::StatusCode;


use crate::web::AppState;

use super::errors::DiscoveryError;

// handle discover feature request
// https://didcomm.org/discover-features/2.0/
pub(crate) fn handle_query_request(
    state: Arc<AppState>,
    message: Message,
) -> Result<Option<Message>, Response> {

    let disclosed_protocols: Vec<String> = Vec::new();
    let supported_protocols: Vec<String> = Vec::new();

    let body = message.body.get("queries");

    if body.is_none() {
        return  Err(StatusCode::BAD_REQUEST.into_response());
    }

    let querie = body.unwrap().as_array();
    if querie.is_none() {
        return Err(DiscoveryError::MalformedBody.json().into_response());
    }
    let queries = querie.unwrap();

  for sq in queries {
    let val = sq.get("match");
    // only supports feature-type protocol
    match val {
        Some(val) => {
          let protocol = val.to_string();
          if supported_protocols.contains(&protocol) {
            disclosed_protocols.push(protocol);
          };
          // build response message
          

        },
        None => Err(DiscoveryError::MalformedBody.json().into_response())
    }
  }
}
