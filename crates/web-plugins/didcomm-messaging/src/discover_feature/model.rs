use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Queries {
    queries: Vec<Value>,
}

#[derive(Deserialize)]
pub struct Disclosures {
    disclosures: Vec<DisclosuresContent>,
}
#[derive(Deserialize)]
struct DisclosuresContent {
    #[serde(rename = "feature-type")]
    feature_type: String,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<String>>
}