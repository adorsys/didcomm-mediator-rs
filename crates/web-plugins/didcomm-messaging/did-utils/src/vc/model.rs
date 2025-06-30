use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{didcore::Proofs, ldmodel::Context};

/// Represents a Verifiable Credential.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    /// The @context property defines the vocabulary used in the JSON-LD document.
    pub context: Context,

    /// Identifier of this credential.
    /// WARNING: This is not the identifier of the subject of the credential.
    // Optional globally unique identifiers enable
    // others to express statements about the same thing
    // https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The type of this credential.
    #[serde(rename = "type")]
    pub cred_type: Vec<String>,

    /// The issuer of this credential.
    pub issuer: Issuers,

    /// The date and time the proof was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The date and time the proof expires
    pub valid_until: Option<DateTime<Utc>>,

    /// The credential subject
    pub credential_subject: CredentialSubject,

    /// laguage tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Names>,

    /// text direction string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Descriptions>,

    // === Properties Map===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    /// Dynamic properties of the credential
    pub additional_properties: Option<HashMap<String, Value>>,

    /// Set of proofs
    // We allow a vc to created without the proof block.
    // Event though it is required. As we want to produce
    // the unsecured vesion before proof production or proof
    // verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<Proofs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The credential status
    pub credential_status: Option<CredentialStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The credential schema
    pub credential_schemas: Option<CredentialSchemas>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The related resource
    pub related_resource: Option<Vec<RelatedResource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The refresh service
    pub refresh_service: Option<RefreshService>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
/// The issuers of the credential.
pub enum Issuers {
    Single(Box<Issuer>),
    SetOf(Vec<Issuer>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
/// The issuer of the credential.
pub enum Issuer {
    SingleString(String),
    IssuerObject,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
/// The issuer object
pub struct IssuerObject {
    /// The identifier of the issuer
    pub id: String,
    /// The laguage tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Names>,
    /// The text direction string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Descriptions>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CredentialSubjects {
    Single(Box<CredentialSubject>),
    SetOf(Vec<CredentialSubject>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// The credential subject
pub struct CredentialSubject {
    /// Identifies the subject of the verifiable credential
    /// (the thing the claims are about) and
    /// uses a decentralized identifier, also known as a DID
    // see https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    // === Properties Map===
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    /// Dynamic properties
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Names {
    Single(Box<Name>),
    SetOf(Vec<Name>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Name {
    SingleString(String),
    NameObject,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
/// The name object
pub struct NameObject {
    /// The name
    pub value: String,
    /// The laguage tag
    // see https://www.rfc-editor.org/rfc/rfc5646
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The text direction string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Descriptions {
    Single(Box<Description>),
    SetOf(Vec<Description>),
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Description {
    SingleString(String),
    DescriptionObject,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
/// The description object
pub struct DescriptionObject {
    /// The description
    pub value: String,
    /// The laguage tag
    // see https://www.rfc-editor.org/rfc/rfc5646
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The text direction string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The credential status
pub struct CredentialStatus {
    /// The identifier of the credential status
    pub id: String,

    /// The type of the credential status
    #[serde(rename = "type")]
    pub status_type: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The purpose of the credential
    pub status_purpose: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The index of the status
    pub status_list_index: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The index of the credential status
    pub status_list_credential: Option<String>,
}

// The value of the credentialSchema property MUST be one or more data schemas
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CredentialSchemas {
    Single(Box<CredentialSchema>),
    SetOf(Vec<CredentialSchema>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// The credential schema
pub struct CredentialSchema {
    #[serde(rename = "@context")]
    /// The @context property defines the vocabulary used in the JSON-LD document.
    pub context: Context,

    /// See <https://www.w3.org/TR/vc-data-model-2.0/#identifiers>
    pub id: String,

    /// see <https://www.w3.org/TR/vc-data-model-2.0/#types>
    #[serde(rename = "type")]
    pub schema_type: String,
}

// see https://www.w3.org/TR/vc-data-model-2.0/#integrity-of-related-resources
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Represents the integrity of related resources
pub struct RelatedResource {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@context")]
    /// The @context property defines the vocabulary used in the JSON-LD document.
    pub context: Option<Context>,

    /// The identifier for the resource
    pub id: String,

    #[serde(rename = "digestSRI")]
    /// Cryptographic digest
    pub digest_sri: Option<String>,

    /// Cryptographic digest
    pub digest_multibase: Option<String>,

    /// The media type of the resource
    pub media_type: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents the refresh service
pub struct RefreshService {
    /// The identifier
    pub id: String,

    /// The type of the service
    #[serde(rename = "type")]
    pub rs_type: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Represents a verifiable presentation as defined [here].
///
/// [here]: https://www.w3.org/TR/vc-data-model-2.0/#verifiable-presentations
pub struct VerifiablePresentation {
    #[serde(rename = "@context")]
    /// The @context property defines the vocabulary used in the JSON-LD document.
    pub context: Context,

    /// Optional globally unique identifiers enable
    /// others to express statements about the same thing.
    // https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The list of verifiable credentials
    pub verifiable_credential: Vec<VerifiableCredential>,

    /// Used to express the type of verifiable presentation
    #[serde(rename = "type")]
    pub pres_type: Vec<String>,

    /// Identifies the presenter
    // https://www.w3.org/TR/vc-data-model-2.0/#identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,

    /// Set of proofs
    // We allow a VP to created without the proof block.
    // Event though it is required. As we want to produce
    // the unsecured vesion before proof production or proof
    // verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<Proofs>,
}

// testing
#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use multibase::Base;
    use serde_json::{from_str, json};

    use crate::{
        crypto::{Ed25519KeyPair, Generate},
        proof::{CryptoProof, EdDsaJcs2022, Proof, UnsecuredDocument},
    };

    use super::*;

    #[test]
    fn test_vc() {
        let vc = make_vc(&pub_key_multibase(subject_key_pair()));
        let vc_expected = r#"{"@context":["https://www.w3.org/ns/credentials/v2","https://www.w3.org/ns/credentials/examples/v2"],"credentialSubject":{"alumniOf":{"id":"did:key#z38w6kKWT7hesyxuuVUSH4LsxbcRof4ra1QBDtR1qrc1q","name":"Example University"},"id":"did:key#z38w6kKWT7hesyxuuVUSH4LsxbcRof4ra1QBDtR1qrc1q"},"description":"Graduated from Example University","id":"http://university.example/credentials/3732","issuer":"did:key#z7dNyxjs9BUfsbX11VG4BGDMB3Wg1Pq2NqhSwTBT8UuRC","name":"Jayden Doe","type":["VerifiableCredential","AlumniCredential"],"validFrom":"2023-03-05T19:23:24Z","validUntil":"2023-12-31T19:23:24Z"}"#;
        let vc_canon = json_canon::to_string(&vc).unwrap();
        assert_eq!(vc_expected, vc_canon);
    }

    #[test]
    fn test_vp() {
        let vc = make_vc(&pub_key_multibase(subject_key_pair()));
        let vp = make_vp(vc);
        let vp_expected = r#"{"@context":["https://www.w3.org/ns/credentials/v2","https://www.w3.org/ns/credentials/examples/v2"],"holder":"did:example:123","id":"http://example.edu/credentials/3732","type":["VerifiablePresentation"],"verifiableCredential":[{"@context":["https://www.w3.org/ns/credentials/v2","https://www.w3.org/ns/credentials/examples/v2"],"credentialSubject":{"alumniOf":{"id":"did:key#z38w6kKWT7hesyxuuVUSH4LsxbcRof4ra1QBDtR1qrc1q","name":"Example University"},"id":"did:key#z38w6kKWT7hesyxuuVUSH4LsxbcRof4ra1QBDtR1qrc1q"},"description":"Graduated from Example University","id":"http://university.example/credentials/3732","issuer":"did:key#z7dNyxjs9BUfsbX11VG4BGDMB3Wg1Pq2NqhSwTBT8UuRC","name":"Jayden Doe","type":["VerifiableCredential","AlumniCredential"],"validFrom":"2023-03-05T19:23:24Z","validUntil":"2023-12-31T19:23:24Z"}]}"#;
        let vp_canon = json_canon::to_string(&vp).unwrap();
        assert_eq!(vp_expected, vp_canon);
    }

    #[test]
    fn test_vc_with_proof() {
        let vc = make_vc(&pub_key_multibase(subject_key_pair()));
        let key_pair = issuer_key_pair();
        let public_key = &key_pair.public_key.clone();
        let public_key_multibase = multibase::encode(Base::Base58Btc, public_key.as_bytes());
        let issuer_vm = format!("did:key#{public_key_multibase}");
        let ed_dsa_jcs_2022_prover = EdDsaJcs2022 {
            proof: make_proof(&issuer_vm),
            key_pair,
            proof_value_codec: Some(Base::Base58Btc),
        };
        let vc_json_value = json!(vc);
        let secured_proof = ed_dsa_jcs_2022_prover.proof(vc_json_value.clone()).unwrap();
        let secured_doc = UnsecuredDocument {
            content: vc_json_value,
            proof: crate::proof::Proofs::SingleProof(Box::new(secured_proof)),
        };
        let vc_canon = json_canon::to_string(&secured_doc).unwrap();
        assert_eq!(SECURED_VC, vc_canon);
    }

    #[test]
    fn test_vc_verify_vc_proof() {
        // test parse string into a VerifiableCredential struct
        let secured_vc: VerifiableCredential = from_str(SECURED_VC).unwrap();

        // Test serialize VerifiableCredential struct into a JSON value
        let secure_doc_json_value: Value = serde_json::to_value(&secured_vc).expect("Failed to serialize");

        let prf = secured_vc.proof.unwrap();
        let secured_proof = match prf {
            Proofs::SingleProof(p) => p,
            Proofs::SetOfProofs(_) => panic!("Expected SingleProof"),
        };

        // We are not doing any did based key resolution in this module.
        // we just encoded the public key as a multibase string in the verification method.
        let vm = secured_proof.verification_method.clone();
        let last_index = vm.rfind('#').unwrap();
        let public_key_multibase = &vm[last_index + 1..];
        let public_key_bytes_vector = multibase::decode(public_key_multibase).unwrap();

        // Initialize with zeros or any default value
        let mut public_key_bytes_slice: [u8; 32] = [0; 32];
        public_key_bytes_slice.copy_from_slice(public_key_bytes_vector.1.as_slice());
        let ed_dsa_jcs_2022_verifier = EdDsaJcs2022 {
            proof: *secured_proof,
            key_pair: Ed25519KeyPair::from_public_key(&public_key_bytes_slice).unwrap(),
            proof_value_codec: None,
        };

        ed_dsa_jcs_2022_verifier.verify(secure_doc_json_value).unwrap();
    }

    #[test]
    fn test_vp_with_proof() {
        // test parse string into a VerifiableCredential struct
        let secured_vc: VerifiableCredential = from_str(SECURED_VC).unwrap();
        let vp = make_vp(secured_vc);
        let vp_canon = json_canon::to_string(&vp).unwrap();
        println!("{vp_canon}");
    }

    const CONTEXTS: &[&str] = &["https://www.w3.org/ns/credentials/v2", "https://www.w3.org/ns/credentials/examples/v2"];

    fn make_context() -> Context {
        let mut contexts: Vec<String> = Vec::new();
        for c in CONTEXTS {
            contexts.push(c.to_string());
        }
        Context::SetOfString(contexts)
    }

    fn make_vc(subject_public_key_multibase: &str) -> VerifiableCredential {
        let issuer_key_pair = issuer_key_pair();
        let issuer_pub_key_multibase = pub_key_multibase(issuer_key_pair);

        // Prepare the alumni credential to be added as dynamic property.
        let mut alumni_of_map = HashMap::new();
        alumni_of_map.insert("name".to_string(), Value::String("Example University".to_string()));
        let did = format!("did:key#{subject_public_key_multibase}");
        alumni_of_map.insert("id".to_string(), Value::String(did.to_string()));
        let alumni_of_json_value = json!(alumni_of_map);
        let mut additional_properties: HashMap<String, Value> = HashMap::new();
        additional_properties.insert("alumniOf".to_string(), alumni_of_json_value);
        VerifiableCredential {
            context: make_context(),
            id: Some("http://university.example/credentials/3732".to_string()),
            cred_type: vec!["VerifiableCredential".to_string(), "AlumniCredential".to_string()],
            issuer: Issuers::Single(Box::new(Issuer::SingleString(format!("did:key#{issuer_pub_key_multibase}")))),
            valid_from: Some(Utc.with_ymd_and_hms(2023, 3, 5, 19, 23, 24).unwrap()),
            valid_until: Some(Utc.with_ymd_and_hms(2023, 12, 31, 19, 23, 24).unwrap()),
            credential_subject: CredentialSubject {
                id: Some(did),
                additional_properties: Some(additional_properties),
            },
            name: Some(Names::Single(Box::new(Name::SingleString("Jayden Doe".to_string())))),
            description: Some(Descriptions::Single(Box::new(Description::SingleString(
                "Graduated from Example University".to_string(),
            )))),
            additional_properties: None,
            proof: None,
            credential_status: None,
            credential_schemas: None,
            related_resource: None,
            refresh_service: None,
        }
    }

    fn make_vp(vc: VerifiableCredential) -> VerifiablePresentation {
        VerifiablePresentation {
            context: make_context(),
            id: Some("http://example.edu/credentials/3732".to_string()),
            verifiable_credential: vec![vc],
            pres_type: vec!["VerifiablePresentation".to_string()],
            holder: Some("did:example:123".to_string()),
            proof: None,
        }
    }

    fn issuer_key_pair() -> Ed25519KeyPair {
        make_key_pair("Seed phrase for issuer thirty2!b")
    }

    fn subject_key_pair() -> Ed25519KeyPair {
        make_key_pair("Seed phrase for subject thrty2!b")
    }

    fn make_key_pair(seed_phrase: &str) -> Ed25519KeyPair {
        // let my_string = String::from("Sample seed bytes of thirtytwo!b");
        let seed: &[u8] = seed_phrase.as_bytes();
        Ed25519KeyPair::new_with_seed(seed).unwrap()
    }

    fn pub_key_multibase(key_pair: Ed25519KeyPair) -> String {
        let public_key = &key_pair.public_key.clone();
        multibase::encode(Base::Base58Btc, public_key.as_bytes())
    }

    fn make_proof(issuer_vm: &str) -> Proof {
        Proof {
            id: None,
            proof_type: "DataIntegrityProof".to_string(),
            cryptosuite: Some("jcs-eddsa-2022".to_string()),
            proof_purpose: "assertionMethod".to_string(),
            verification_method: issuer_vm.to_string(),
            created: Some(chrono::Utc.with_ymd_and_hms(2023, 3, 5, 19, 23, 24).unwrap()),
            expires: None,
            domain: Some(crate::proof::Domain::SingleString("vc-demo.adorsys.com".to_string())),
            challenge: Some("523452345234asfdasdfasdfa".to_string()),
            proof_value: None,
            previous_proof: None,
            nonce: Some("1234567890".to_string()),
        }
    }

    const SECURED_VC: &str = r#"{"@context":["https://www.w3.org/ns/credentials/v2","https://www.w3.org/ns/credentials/examples/v2"],"credentialSubject":{"alumniOf":{"id":"did:key#z38w6kKWT7hesyxuuVUSH4LsxbcRof4ra1QBDtR1qrc1q","name":"Example University"},"id":"did:key#z38w6kKWT7hesyxuuVUSH4LsxbcRof4ra1QBDtR1qrc1q"},"description":"Graduated from Example University","id":"http://university.example/credentials/3732","issuer":"did:key#z7dNyxjs9BUfsbX11VG4BGDMB3Wg1Pq2NqhSwTBT8UuRC","name":"Jayden Doe","proof":{"challenge":"523452345234asfdasdfasdfa","created":"2023-03-05T19:23:24Z","cryptosuite":"eddsa-jcs-2022","domain":"vc-demo.adorsys.com","nonce":"1234567890","proofPurpose":"assertionMethod","proofValue":"z4DEMwgRCZnRddGPPevbaafihRwj4ng3dn5EwmnnaeMVMp25niKWZ3cW1rdfWMtfp5dpCmNEjfJtvbnnpUsZcy9c6","type":"DataIntegrityProof","verificationMethod":"did:key#z7dNyxjs9BUfsbX11VG4BGDMB3Wg1Pq2NqhSwTBT8UuRC"},"type":["VerifiableCredential","AlumniCredential"],"validFrom":"2023-03-05T19:23:24Z","validUntil":"2023-12-31T19:23:24Z"}"#;
}
