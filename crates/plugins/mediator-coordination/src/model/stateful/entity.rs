use didcomm::{Attachment, Message};
/// Resources to map in a database.
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

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
pub struct Messages {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub message: Vec<Attachment>,
    pub recipient_did: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Secrets {
    #[serde(rename = "_id")]
    pub id: ObjectId,

    pub kid: String,
    pub type_: i32,
    pub verification_material: VerificationMaterial,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerificationMaterial {
    pub format: i32,
    pub value: String,
}
