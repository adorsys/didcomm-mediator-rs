use serde::{Deserialize, Serialize};

use super::error::DIDPeerMethodError;
use crate::{
    crypto::{ed25519::Ed25519KeyPair, sha256_hash::sha256_multihash},
    didcore::Document as DIDDocument,
    methods::{
        common::{Algorithm, PublicKeyFormat},
        did_key::DIDKeyMethod,
        traits::DIDMethod,
    },
};

#[derive(Default)]
pub struct DIDPeerMethod {
    /// Key format to consider during DID expansion into a DID document
    pub key_format: PublicKeyFormat,

    /// Derive key agreement on expanding did:peer:0 address
    pub enable_encryption_key_derivation: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Purpose {
    Assertion,
    Encryption,   // Key Agreement
    Verification, // Authentication
    CapabilityInvocation,
    CapabilityDelegation,
    Service,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PurposedKey {
    pub purpose: Purpose,
    pub public_key_multibase: String,
}

impl DIDMethod for DIDPeerMethod {
    fn name() -> String {
        "did:peer".to_string()
    }
}

impl Purpose {
    /// Converts purpose to normalized one-letter code
    pub fn code(&self) -> char {
        match self {
            Purpose::Assertion => 'A',
            Purpose::Encryption => 'E',
            Purpose::Verification => 'V',
            Purpose::CapabilityInvocation => 'I',
            Purpose::CapabilityDelegation => 'D',
            Purpose::Service => 'S',
        }
    }

    /// Derives purpose from normalized one-letter code
    pub fn from_code(c: &char) -> Result<Self, DIDPeerMethodError> {
        match c {
            'A' => Ok(Purpose::Assertion),
            'E' => Ok(Purpose::Encryption),
            'V' => Ok(Purpose::Verification),
            'I' => Ok(Purpose::CapabilityInvocation),
            'D' => Ok(Purpose::CapabilityDelegation),
            'S' => Ok(Purpose::Service),
            _ => Err(DIDPeerMethodError::InvalidPurposeCode),
        }
    }
}

impl DIDPeerMethod {
    /// Method 0: Generates did:peer address from ed25519 inception key without doc
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-0-inception-key-without-doc
    pub fn create_did_peer_0_from_ed25519_keypair(keypair: &Ed25519KeyPair) -> Result<String, DIDPeerMethodError> {
        let did_key = DIDKeyMethod::from_ed25519_keypair(keypair)?;

        Ok(did_key.replace("did:key:", "did:peer:0"))
    }

    /// Method 0: Generates did:peer address from inception key without doc
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-0-inception-key-without-doc
    pub fn create_did_peer_0_from_raw_public_key(alg: Algorithm, bytes: &[u8]) -> Result<String, DIDPeerMethodError> {
        let did_key = DIDKeyMethod::from_raw_public_key(alg, bytes)?;

        Ok(did_key.replace("did:key:", "did:peer:0"))
    }

    /// Method 1: Generates did:peer address from DID document
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-1-genesis-doc
    pub fn create_did_peer_1_from_diddoc(diddoc: &DIDDocument) -> Result<String, DIDPeerMethodError> {
        if !diddoc.id.is_empty() {
            return Err(DIDPeerMethodError::InvalidStoredVariant);
        }

        let json = json_canon::to_string(diddoc)?;
        let multihash = sha256_multihash(json.as_bytes());

        Ok(format!("did:peer:1z{multihash}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_jwk::jwk::Jwk;

    #[test]
    fn test_did_peer_0_generation_from_given_jwk() {
        let jwk: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "O2onvM62pC1io6jQKm8Nc2UyFXcd4kOmOsBIoYtZ2ik"
            }"#,
        )
        .unwrap();
        let keypair: Ed25519KeyPair = jwk.try_into().unwrap();

        let did = DIDPeerMethod::create_did_peer_0_from_ed25519_keypair(&keypair);
        assert_eq!(did.unwrap(), "did:peer:0z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp");
    }

    #[test]
    fn test_did_peer_0_generation_from_given_raw_public_key_bytes() {
        let entries = [
            (
                Algorithm::Ed25519,
                hex::decode("3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29").unwrap(),
                "did:peer:0z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            ),
            (
                Algorithm::X25519,
                hex::decode("2fe57da347cd62431528daac5fbb290730fff684afc4cfc2ed90995f58cb3b74").unwrap(),
                "did:peer:0z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F",
            ),
        ];

        for entry in entries {
            let (alg, bytes, expected) = entry;
            let did = DIDPeerMethod::create_did_peer_0_from_raw_public_key(alg, &bytes);
            assert_eq!(did.unwrap(), expected);
        }
    }

    #[test]
    fn test_did_peer_1_generation_from_did_document() {
        let diddoc: DIDDocument = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/ed25519-2020/v1"
                ],
                "id": "",
                "verificationMethod": [{
                    "id": "#key1",
                    "type": "Ed25519VerificationKey2020",
                    "controller": "#id",
                    "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
                }],
                "authentication": ["#key1"],
                "assertionMethod": ["#key1"],
                "capabilityDelegation": ["#key1"],
                "capabilityInvocation": ["#key1"]
            }"##,
        )
        .unwrap();

        let did = DIDPeerMethod::create_did_peer_1_from_diddoc(&diddoc);
        assert_eq!(did.unwrap(), "did:peer:1zQmbEB1EqP7PnNVaHiSpXhkatAA6kNyQK9mWkvrMx2eckgq");
    }

    #[test]
    fn test_did_peer_1_generation_fails_from_did_document_with_id() {
        let diddoc: DIDDocument = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/ed25519-2020/v1"
                ],
                "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "verificationMethod": [{
                    "id": "#key1",
                    "type": "Ed25519VerificationKey2020",
                    "controller": "#id",
                    "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
                }],
                "authentication": ["#key1"],
                "assertionMethod": ["#key1"],
                "capabilityDelegation": ["#key1"],
                "capabilityInvocation": ["#key1"]
            }"##,
        )
        .unwrap();

        let did = DIDPeerMethod::create_did_peer_1_from_diddoc(&diddoc);
        assert!(matches!(did.unwrap_err(), DIDPeerMethodError::InvalidStoredVariant));
    }
}
