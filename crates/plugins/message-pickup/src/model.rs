use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct StatusRequest {
    id: String,

    #[serde(rename = "type")]
    type_: String,

    body: BodyResponse,

    return_route: ReturnRoute,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BodyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    recipient_did: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ReturnRoute {
    None,
    Thread,
    #[default]
    All,
}
