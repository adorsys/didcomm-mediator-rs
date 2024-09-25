use database::Identifiable; // Ensure this path is correct
/// Resources to map in a database.
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Record of a mediation relationship between an edge agent (client) and a mediator.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Connection {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Client's DID used at mediation coordination.
    pub client_did: String,

    /// Mediator's DID used at mediation coordination.
    pub mediator_did: String,

    /// List of DIDs maintained by the client in the mediator's
    /// database by a series of keylist operations.
    pub keylist: Vec<String>,

    /// Generated DID to route messages to client.
    pub routing_did: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RoutedMessage {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub message: Value,
    pub recipient_did: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Secrets {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub kid: String,
    pub type_: i32,
    pub verification_material: VerificationMaterial,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerificationMaterial {
    pub format: i32,
    pub value: String,
}

impl Identifiable for Connection {
    fn id(&self) -> Option<ObjectId> {
        self.id
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}

impl Identifiable for RoutedMessage {
    fn id(&self) -> Option<ObjectId> {
        self.id
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}

impl Identifiable for Secrets {
    fn id(&self) -> Option<ObjectId> {
        self.id.clone() // Adjust if making it optional
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}