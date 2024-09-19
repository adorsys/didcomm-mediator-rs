use didcomm::{Attachment, Message, MessageBuilder};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
pub(crate) struct StatusResponse<'a> {
    pub(crate) id: &'a str,

    #[serde(rename = "type")]
    pub(crate) type_: &'a str,

    pub(crate) body: BodyStatusResponse<'a>,
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct BodyStatusResponse<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recipient_did: Option<&'a str>,

    pub(crate) message_count: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) longest_waited_seconds: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) newest_received_time: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) oldest_received_time: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total_bytes: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) live_delivery: Option<bool>,
}

impl<'a> From<StatusResponse<'a>> for MessageBuilder {
    fn from(value: StatusResponse<'a>) -> Self {
        Message::build(
            value.id.to_string(),
            value.type_.to_string(),
            json!(value.body),
        )
    }
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct DeliveryResponse<'a> {
    pub(crate) id: &'a str,

    pub(crate) thid: &'a str,

    #[serde(rename = "type")]
    pub(crate) type_: &'a str,

    pub(crate) body: BodyDeliveryResponse<'a>,

    pub(crate) attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct BodyDeliveryResponse<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recipient_did: Option<&'a str>,
}

impl<'a> From<DeliveryResponse<'a>> for MessageBuilder {
    fn from(value: DeliveryResponse<'a>) -> Self {
        Message::build(
            value.id.to_string(),
            value.type_.to_string(),
            json!(value.body),
        )
        .thid(value.thid.to_string())
        .attachments(value.attachments)
    }
}
