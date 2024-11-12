use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Serialize, Deserialize)]
pub struct BasicMessage {
    /// Message ID (unique identifier)
    pub id: String,
    /// Message type URI for basic messages
    #[serde(rename = "type")]
    pub message_type: String,
    /// Localization information for message language
    #[serde(rename = "lang")]
    pub locale: Option<String>,
    /// Timestamp for when the message was created (DIDComm V2 standard)
    #[serde(rename = "created_time")]
    pub created_time: DateTime<Utc>,
    /// Main message body
    pub body: MessageBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageBody {
    pub content: String,
}
