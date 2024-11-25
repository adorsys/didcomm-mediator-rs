#![cfg_attr(rustfmt, rustfmt_skip)]

pub const MEDIATE_REQUEST_2_0: &str = "https://didcomm.org/coordinate-mediation/2.0/mediate-request";
pub const MEDIATE_DENY_2_0: &str = "https://didcomm.org/coordinate-mediation/2.0/mediate-deny";
pub const MEDIATE_GRANT_2_0: &str = "https://didcomm.org/coordinate-mediation/2.0/mediate-grant";
pub const KEYLIST_UPDATE_2_0: &str = "https://didcomm.org/coordinate-mediation/2.0/keylist-update";
pub const KEYLIST_UPDATE_RESPONSE_2_0: &str = "https://didcomm.org/coordinate-mediation/2.0/keylist-update-response";
pub const KEYLIST_QUERY_2_0: &str = "https://didcomm.org/coordinate-mediation/2.0/keylist-query";
pub const KEYLIST_2_0: &str = "https://didcomm.org/coordinate-mediation/2.0/keylist";
pub const MEDIATE_FORWARD_2_0: &str = "https://didcomm.org/routing/2.0/forward";

pub const MEDIATE_REQUEST_DIC_1_0: &str = "https://didcomm.org/coordinate-mediation/dic/1.0/mediate-request";
pub const MEDIATE_DENY_DIC_1_0: &str = "https://didcomm.org/coordinate-mediation/dic/1.0/mediate-deny";
pub const MEDIATE_GRANT_DIC_1_0: &str = "https://didcomm.org/coordinate-mediation/dic/1.0/mediate-grant";

pub const STATUS_REQUEST_3_0: &str = "https://didcomm.org/messagepickup/3.0/status-request";
pub const STATUS_RESPONSE_3_0: &str = "https://didcomm.org/messagepickup/3.0/status";
pub const DELIVERY_REQUEST_3_0: &str = "https://didcomm.org/messagepickup/3.0/delivery-request";
pub const MESSAGE_DELIVERY_3_0: &str = "https://didcomm.org/messagepickup/3.0/delivery";
pub const MESSAGE_RECEIVED_3_0: &str = "https://didcomm.org/messagepickup/3.0/messages-received";
pub const LIVE_MODE_CHANGE_3_0: &str = "https://didcomm.org/messagepickup/3.0/live-delivery-change";
pub const PROBLEM_REPORT_2_0: &str = "https://didcomm.org/report-problem/2.0/problem-report";

pub const TRUST_PING_2_0: &str = "https://didcomm.org/trust-ping/2.0/ping";
pub const TRUST_PING_RESPONSE_2_0: &str = "https://didcomm.org/trust-ping/2.0/ping-response";

pub const DIDCOMM_ENCRYPTED_MIME_TYPE: &str = "application/didcomm-encrypted+json";
pub const DIDCOMM_ENCRYPTED_SHORT_MIME_TYPE: &str = "didcomm-encrypted+json";
pub const DISCOVER_FEATURE: &str = "https://didcomm.org/discover-features/2.0/disclose";
pub const QUERY_FEATURE: &str = "https://didcomm.org/discover-features/2.0/queries";