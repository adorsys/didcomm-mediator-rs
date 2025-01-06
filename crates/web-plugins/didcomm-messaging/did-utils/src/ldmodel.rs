//! Provides Linked Data models for representing DIDs and related data.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the JSON-LD context.
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
// The @context property defines the vocabulary used in the JSON-LD document.
// It provides a way to map the keys in the JSON structure to specific terms,
// properties, and classes from external vocabularies.
pub enum Context {
    /// A single string value.
    SingleString(String),
    /// A set of string values.
    SetOfString(Vec<String>),
    /// A JSON object.
    JsonObject(Value),
}
