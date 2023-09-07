use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ldmodel::Context, didcore::Proofs};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    pub context: Context,

    // Optional globally unique identifiers enable
    // others to express statements about the same thing
    // https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    // see https://www.w3.org/TR/vc-data-model-2.0/#types
    #[serde(rename = "type")]
    pub cred_type: Vec<String>,

    // see https://www.w3.org/TR/vc-data-model-2.0/#issuer
    pub issuer: Issuers,

    // The date and time the proof was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,

    // See https://www.w3.org/TR/vc-data-model-2.0/#credential-subject
    pub credential_subject: CredentialSubject,

    // laguage tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Names>,

    // text direction string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Descriptions>,

    // === Properties Map===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,

    // Set of proofs
    pub proof: Proofs,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_status: Option<CredentialStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_schemas: Option<CredentialSchemas>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_resource: Option<Vec<RelatedResource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_service: Option<RefreshService>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Issuers {
    Single(Box<Issuer>),
    SetOf(Box<Vec<Issuer>>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Issuer {
    SingleString(String),
    IssuerObject,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
pub struct IssuerObject {
    pub id: String,
    // laguage tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Names>,
    // text direction string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Descriptions>,
}



#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CredentialSubjects {
    Single(Box<CredentialSubject>),
    SetOf(Box<Vec<CredentialSubject>>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSubject {
    // Identifies the subject of the verifiable credential 
    // (the thing the claims are about) and 
    // uses a decentralized identifier, also known as a DID
    // see https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    // === Properties Map===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Names {
    Single(Box<Name>),
    SetOf(Box<Vec<Name>>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Name {
    SingleString(String),
    NameObject,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
pub struct NameObject {
    pub value: String,
    // laguage tag
    // see https://www.rfc-editor.org/rfc/rfc5646
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    // text direction string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Descriptions {
    Single(Box<Description>),
    SetOf(Box<Vec<Description>>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Description {
    SingleString(String),
    DescriptionObject,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
pub struct DescriptionObject {
    pub value: String,
    // laguage tag
    // see https://www.rfc-editor.org/rfc/rfc5646
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    // text direction string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {

    pub id: String,

    // see https://www.w3.org/TR/vc-data-model-2.0/#types
    #[serde(rename = "type")]
    pub status_type: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_purpose: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_list_index: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_list_credential: Option<String>,

}

// The value of the credentialSchema property MUST be one or more data schemas
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CredentialSchemas {
    Single(Box<CredentialSchema>),
    SetOf(Box<Vec<CredentialSchema>>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSchema {
    #[serde(rename = "@context")]
    pub context: Context,

    // https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    pub id: String,

    // see https://www.w3.org/TR/vc-data-model-2.0/#types
    #[serde(rename = "type")]
    pub schema_type: String,
}

// see https://www.w3.org/TR/vc-data-model-2.0/#integrity-of-related-resources
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelatedResource {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@context")]
    pub context: Option<Context>,

    // https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    pub id: String,

    #[serde(rename = "digestSRI")]
    pub digest_sri: Option<String>,

    pub digest_multibase: Option<String>,

    pub media_type: Option<String>
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshService {

    pub id: String,

    // see https://www.w3.org/TR/vc-data-model-2.0/#types
    #[serde(rename = "type")]
    pub rs_type: String,
}



#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiablePresentation {
    #[serde(rename = "@context")]
    pub context: Context,

    // Optional globally unique identifiers enable
    // others to express statements about the same thing
    // https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    pub verifiable_credential: Vec<VerifiableCredential>,

    // see https://www.w3.org/TR/vc-data-model-2.0/#types
    #[serde(rename = "type")]
    pub pres_type: Vec<String>,

    // Identifies the presenter
    // https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,

    // Set of proofs
    pub proof: Proofs,
}

