use crate::model::{BasicMessage, MessageBody};
use crate::error::ProtocolError;
use chrono::Utc;
use uuid::Uuid;

///creating a new basic message
pub fn basic_message(content: &str, lang: Option<String>) -> Result<BasicMessage, ProtocolError> {
    Ok(BasicMessage {
        id: Uuid::new_v4().to_string(),
        message_type: "https://didcomm.org/basicmessage/2.0/message".to_string(),
        locale: lang,
        created_time: Utc::now(),
        body: MessageBody {
            content: content.to_string(),
        },
    })
}

/// Function to handle the received message 
pub fn handle_received_message(message: &BasicMessage) -> Result<(), ProtocolError> {
    println!("Received message: {}", message.body.content);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::handler::{basic_message, handle_received_message};

    #[test]
    fn test_create_and_handle_basic_message() {
        let message_content = "Hello, this is a basic message.";
        let message = basic_message(message_content, Some("en".to_string())).unwrap();
        assert_eq!(message.body.content, message_content);

        handle_received_message(&message).expect("Failed to handle received message");
    }
}