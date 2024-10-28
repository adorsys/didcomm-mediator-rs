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
            value.id.to_owned(),
            value.type_.to_owned(),
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
            value.id.to_owned(),
            value.type_.to_owned(),
            json!(value.body),
        )
        .thid(value.thid.to_owned())
        .attachments(value.attachments)
    }
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct LiveDeliveryChange<'a> {
    pub(crate) id: &'a str,

    pub(crate) pthid: &'a str,
    
    #[serde(rename = "type")]
    pub(crate) type_: &'a str,

    pub(crate) body: BodyLiveDeliveryChange<'a>,
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct BodyLiveDeliveryChange<'a> {
    pub(crate) code: &'a str,

    pub(crate) comment: &'a str,
}

impl<'a> From<LiveDeliveryChange<'a>> for MessageBuilder {
    fn from(value: LiveDeliveryChange<'a>) -> Self {
        Message::build(
            value.id.to_owned(),
            value.type_.to_owned(),
            json!(value.body),
        )
        .pthid(value.pthid.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use didcomm::{MessageBuilder, Attachment};
    use serde_json::{json, Value};

    #[test]
    fn test_status_response_serialization() {
        let id = "123456789";
        let type_ = "https://didcomm.org/messagepickup/3.0/status-response";
        let response = StatusResponse {
            id,
            type_,
            body: BodyStatusResponse {
                recipient_did: Some("did:example:recipient"),
                message_count: 5,
                longest_waited_seconds: Some(160),
                live_delivery: Some(false),
                ..Default::default()
            },
        };

        let message_builder: MessageBuilder = response.into();
        let message = message_builder.finalize();
        let expected_body = json!({
            "recipient_did": "did:example:recipient",
            "message_count": 5,
            "longest_waited_seconds": 160,
            "live_delivery": false
        });

        assert_eq!(message.id, id);
        assert_eq!(message.type_, type_);
        assert_eq!(message.body, expected_body);
    }

    #[test]
    fn test_delivery_response_serialization() {
        let id = "12345";
        let thid = "67890";
        let type_ = "https://didcomm.org/messagepickup/3.0/delivery-response";
        let attachment = Attachment::json(json!({"key": "value"})).id("123".to_owned()).finalize();
        let response = DeliveryResponse {
            id,
            thid,
            type_,
            body: BodyDeliveryResponse { recipient_did: None },
            attachments: vec![attachment.clone()],
        };

        let message_builder: MessageBuilder = response.into();
        let message = message_builder.finalize();

        assert_eq!(message.id, id);
        assert_eq!(message.thid, Some(thid.to_owned()));
        assert_eq!(message.type_, type_);
        assert_eq!(message.body["recipient_did"], Value::Null);
        assert_eq!(message.attachments.as_ref().unwrap().len(), 1);
        assert_eq!(message.attachments.as_ref().unwrap()[0], attachment);
        assert_eq!(message.attachments, Some(vec![attachment]));
    }

    #[test]
    fn test_live_delivery_change_serialization() {
        let id = "12345";
        let pthid = "67890";
        let type_ = "https://didcomm.org/messagepickup/3.0/live-delivery-change";
        let response = LiveDeliveryChange {
            id,
            pthid,
            type_,
            body: BodyLiveDeliveryChange {
                code: "code123",
                comment: "just some weird error comment",
            },
        };

        let message_builder: MessageBuilder = response.into();
        let message = message_builder.finalize();
        let expected_body = json!({
            "code": "code123",
            "comment": "just some weird error comment"
        });

        assert_eq!(message.id, id);
        assert_eq!(message.pthid, Some(pthid.to_owned()));
        assert_eq!(message.type_, type_);
        assert_eq!(message.body, expected_body);
    }
}
