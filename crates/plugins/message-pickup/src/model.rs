use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct StatusRequest<'a> {
    pub(crate) id: &'a str,

    #[serde(rename = "type")]
    pub(crate) type_: &'a str,

    pub(crate) body: BodyStatusRequest<'a>,

    pub(crate) return_route: ReturnRoute,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BodyStatusRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    recipient_did: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StatusResponse<'a> {
    pub(crate) id: &'a str,

    #[serde(rename = "type")]
    pub(crate) type_: &'a str,

    pub(crate) body: BodyStatusResponse<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct BodyStatusResponse<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recipient_did: Option<&'a str>,

    pub(crate) message_count: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) longest_waited_seconds: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lnewest_received_time: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) oldest_received_time: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total_bytes: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) live_delivery: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ReturnRoute {
    None,
    Thread,
    #[default]
    All,
}
