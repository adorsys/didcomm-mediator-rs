use std::sync::Arc;
use axum::response::Response;
use chrono::Utc;
use serde_json::json;
use shared::state::AppState;
use uuid::Uuid;
use crate::model::BasicMessage;

pub async fn handle_basic_message(
    state: Arc<AppState>,
    message: BasicMessage,
) -> Result<Option<BasicMessage>, Response> {
    // Create the response message based on the input message
    let response_message = BasicMessage {
        id: Uuid::new_v4().to_string(),
        message_type: "https://didcomm.org/basicmessage/2.0/message".to_string(),
        lang: message.lang.clone(),
        created_time: Utc::now(),
        body: json!({ "content": format!("Received: {}", message.body["content"].as_str().unwrap_or("")) }),
    };

    Ok(Some(response_message))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BasicMessage;
    use chrono::Utc;
    use did_utils::didcore::Document;
    use serde_json::json;
    use shared::state::AppState;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_handle_basic_message() {
        let app_state = Arc::new(AppState::from("".to_string(), Document::default(), None));

        let input_message = BasicMessage {
            id: Uuid::new_v4().to_string(),
            message_type: "https://didcomm.org/basicmessage/2.0/message".to_string(),
            lang: Some("en".to_string()),
            created_time: Utc::now(),
            body: json!({ "content": "Hello, this is a test message." }),
        };

        let result = handle_basic_message(app_state, input_message.clone()).await;

        assert!(result.is_ok());
        let response_message = result.unwrap().unwrap();

        assert_eq!(response_message.message_type, "https://didcomm.org/basicmessage/2.0/message");
        assert_eq!(response_message.lang, input_message.lang);
        assert_eq!(response_message.body["content"], json!(format!("Received: {}", input_message.body["content"].as_str().unwrap())));

        assert_ne!(response_message.id, input_message.id);
        assert!(response_message.created_time >= input_message.created_time);
    }
}
