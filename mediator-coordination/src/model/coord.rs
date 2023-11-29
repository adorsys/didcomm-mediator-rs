use serde::{Deserialize, Serialize};

use super::stateless::coord::MediationRequest as StatelessMediationRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MediationRequest {
    /// Format for stateful standard mode
    // Stateful

    /// Format for stateless mode over DICs
    Stateless(StatelessMediationRequest),
}
