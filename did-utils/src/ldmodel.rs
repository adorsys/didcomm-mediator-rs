use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
    // The @context property defines the vocabulary used in the JSON-LD document.
    // It provides a way to map the keys in the JSON structure to specific terms,
    // properties, and classes from external vocabularies.
pub enum Context {
    SingleString(String),
    SetOfString(Vec<String>),
}
