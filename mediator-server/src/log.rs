use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Error, Result, ClientError};
use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

#[allow(dead_code)]
pub async fn log_request(
    uuid: Uuid,
    req_method: Method,
    uri: Uri,
    service_error: Option<&Error>,
    client_error: Option<ClientError>,
) -> Result<()> {
    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis();

    let error_type = service_error.map(|se| se.as_ref().to_string());
    let error_data = serde_json::to_value(service_error)
        .ok()
        .and_then(|mut v| v.get_mut("data").map(|v| v.take()));

    // Create the RequestLogLine
    let log_line = RequestLogLine {
        uuid: uuid.to_string(),
        timestamp: timestamp.to_string(),

        req_path: uri.to_string(),
        req_method: req_method.to_string(),
        
        client_error_type: client_error.map(|ce| ce.as_ref().to_string()),

        error_type,
        error_data,
    };
    
    println!("    ->> log_request: \n{}", json!(log_line));

    // TODO - send to cloud-watch

    Ok(())
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
    uuid: String, // uuid string formatted
    timestamp: String, // should be iso8601

    // -- Request Attributes
    req_path: String,
    req_method: String,

    // -- Errors Attributes
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,
}