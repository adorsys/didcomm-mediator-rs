use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicMessage {
    /// Message ID (unique identifier)
    pub id: String,
    /// Message type URI for basic messages
    pub message_type: String,
    /// Localization information for message language
    pub lang: Option<String>,
    /// Timestamp for when the message was created (DIDComm V2 standard)
    pub created_time: DateTime<Utc>,
    /// Main message body
    pub body: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageBody {
    pub content: String,
}
