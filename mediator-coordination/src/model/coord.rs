use did_utils::vc::model::VerifiablePresentation;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

use super::dic::CompactDIC;

/// Types of services a mediator can offer to a registered edge agent.
/// - Inbox: Receive and store messages intended for an edge agent for eventual pickup.
/// - Outbox: Relay a message in the SSI network.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum MediatorService {
    Inbox,
    Outbox,
}

/// Message for mediation request.
///
/// It conveys key parameters as an edge agent requests mediation
/// from a cloud agent, hereinafter mediator. It includes details
/// such as the  range of services requested from the mediator and
/// a cryptographic means to ensure secure further communication
/// with the edge agent and to verify its digital signatures.
///
/// To a mediation request is expected a grant or a deny response.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediationRequest {
    /// Uniquely identifies a mediation request message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/mediate-request`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Edge agent's decentralized identifier.
    ///
    /// From this, the mediator MUST be able to derive crypto keys to
    /// enable encrypted peer communication and signature verification.
    pub did: String,

    /// Services requested from the mediator.
    pub services: BTreeSet<MediatorService>,

    /// Business-defined presentation to be verified
    /// by the mediator to avoid spamming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anti_spam: Option<VerifiablePresentation>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Message for mediation grant.
///
/// It conveys a positive response from the mediator to a mediation request,
/// carrying details the edge agent will be responsible to advertise including
/// assertions for dedicated interaction channels (DICs).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediationGrant {
    /// Uniquely identifies a mediation grant message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/mediate-grant`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Mediator's endpoint.
    pub endpoint: String,

    /// DICs (Dedicated Interaction Channels)
    ///
    /// They represent on their own a proof of authorized interaction
    /// delivered by the mediator according to an edge agent's request.
    pub dic: Vec<CompactDIC>,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

/// Message for mediation deny.
///
/// It conveys a negative response from the mediator to a mediation request.
/// This can be issued for several reasons, including business-specific ones.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediationDeny {
    /// Uniquely identifies a mediation deny message.
    #[serde(rename = "@id")]
    pub id: String,

    /// References the protocol URI of this concept.
    ///
    /// Typically `https://didcomm.org/coordinate-mediation/2.0/mediate-deny`
    #[serde(rename = "@type")]
    pub message_type: String,

    /// Dynamic properties.
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}
