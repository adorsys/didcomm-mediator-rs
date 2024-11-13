use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicMessage {
    /// Message ID (unique identifier)
    pub id: String,
    /// Message type URI for basic messages
    pub message_type: String,
    /// Localization information for message language
    pub locale: Option<String>,
    /// Timestamp for when the message was created (DIDComm V2 standard)
    pub created_time: DateTime<Utc>,
    /// Main message body
    pub body: MessageBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageBody {
    pub content: String,
}
