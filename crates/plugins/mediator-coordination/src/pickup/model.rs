use serde::Serialize;
#[derive(Debug, Serialize)]
pub(crate) struct StatusResponse<'a> {
    pub(crate) id: &'a str,

    #[serde(rename = "type")]
    pub(crate) type_: &'a str,

    pub(crate) body: BodyStatusResponse<'a>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) struct BodyStatusResponse<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recipient_did: Option<&'a str>,

    pub(crate) message_count: usize,

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
