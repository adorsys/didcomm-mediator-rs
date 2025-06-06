use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
pub struct Queries {
    pub queries: Vec<Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Disclosures {
    pub disclosures: Vec<Value>,
}
impl Disclosures {
    pub fn new() -> Self {
        Disclosures {
            disclosures: vec![],
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct DisclosuresContent {
    #[serde(rename = "feature-type")]
    pub feature_type: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}
