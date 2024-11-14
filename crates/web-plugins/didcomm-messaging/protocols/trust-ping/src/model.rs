use didcomm::{Message, MessageBuilder};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub(crate) struct TrustPingResponse<'a> {
    pub(crate) id: &'a str,

    #[serde(rename = "type")]
    pub(crate) type_: &'a str,

    pub(crate) thid: &'a str,
}

impl<'a> From<TrustPingResponse<'a>> for MessageBuilder {
    fn from(value: TrustPingResponse<'a>) -> Self {
        Message::build(
            value.id.to_owned(),
            value.type_.to_owned(),
            json!(Value::Null),
        )
        .thid(value.thid.to_owned())
    }
}
