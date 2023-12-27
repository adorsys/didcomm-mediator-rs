use multibase::Base::Base64Url;
use serde::{Deserialize, Serialize};

use super::{error::DIDPeerMethodError, util::abbreviate_service_for_did_peer_2};
use crate::{
    crypto::{ed25519::Ed25519KeyPair, sha256_hash::sha256_multihash},
    didcore::{Document as DIDDocument, Service},
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
    pub fn create_did_peer_1_from_stored_variant(diddoc: &DIDDocument) -> Result<String, DIDPeerMethodError> {
        if !diddoc.id.is_empty() {
            return Err(DIDPeerMethodError::InvalidStoredVariant);
        }

        let json = json_canon::to_string(diddoc)?;
        let multihash = sha256_multihash(json.as_bytes());

        Ok(format!("did:peer:1z{multihash}"))
    }

    /// Method 2: Generates did:peer address from multiple inception key
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-2-multiple-inception-key-without-doc
    pub fn create_did_peer_2(keys: &[PurposedKey], services: &[Service]) -> Result<String, DIDPeerMethodError> {
        if keys.is_empty() && services.is_empty() {
            return Err(DIDPeerMethodError::EmptyArguments);
        }

        // Initialization
        let mut chain = vec![];

        // Chain keys
        for key in keys {
            if matches!(key.purpose, Purpose::Service) {
                return Err(DIDPeerMethodError::UnexpectedPurpose);
            }

            chain.push(format!(".{}{}", key.purpose.code(), key.public_key_multibase));
        }

        // Chain services
        for service in services {
            let abbreviated_service = abbreviate_service_for_did_peer_2(service)?;
            let encoded_service = Base64Url.encode(abbreviated_service);

            chain.push(format!(".{}{}", Purpose::Service.code(), encoded_service));
        }

        Ok(format!("did:peer:2{}", chain.join("")))
    }

    /// Method 3: DID Shortening with SHA-256 Hash
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-3-did-shortening-with-sha-256-hash
    pub fn create_did_peer_3(did: &str) -> Result<String, DIDPeerMethodError> {
        let stripped = match did.strip_prefix("did:peer:2") {
            Some(stripped) => stripped,
            None => return Err(DIDPeerMethodError::IllegalArgument),
        };

        // Multihash with SHA256
        let multihash = sha256_multihash(stripped.as_bytes());

        Ok(format!("did:peer:3z{multihash}"))
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

        let did = DIDPeerMethod::create_did_peer_1_from_stored_variant(&diddoc);
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

        let did = DIDPeerMethod::create_did_peer_1_from_stored_variant(&diddoc);
        assert!(matches!(did.unwrap_err(), DIDPeerMethodError::InvalidStoredVariant));
    }

    #[test]
    fn test_did_peer_2_generation() {
        let keys: Vec<PurposedKey> = serde_json::from_str(
            r##"[
                {
                    "purpose": "verification",
                    "publicKeyMultibase": "z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"
                },
                {
                    "purpose": "encryption",
                    "publicKeyMultibase": "z6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR"
                }
            ]"##,
        )
        .unwrap();

        let did = DIDPeerMethod::create_did_peer_2(&keys, &[]).unwrap();
        assert_eq!(
            &did,
            "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR"
        );
    }

    #[test]
    fn test_did_peer_2_generation_with_service() {
        let keys: Vec<PurposedKey> = serde_json::from_str(
            r##"[{
                "purpose": "verification",
                "publicKeyMultibase": "z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"
            }]"##,
        )
        .unwrap();

        let services = vec![Service {
            id: String::from("#didcomm"),
            service_type: String::from("DIDCommMessaging"),
            service_endpoint: String::from("http://example.com/didcomm"),
            additional_properties: None,
        }];

        assert_eq!(
            &DIDPeerMethod::create_did_peer_2(&keys, &services).unwrap(),
            concat!(
                "did:peer:2",
                ".Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc",
                ".SeyJpZCI6IiNkaWRjb21tIiwicyI6Imh0dHA6Ly9leGFtcGxlLmNvbS9kaWRjb21tIiwidCI6ImRtIn0"
            )
        );
    }

    #[test]
    fn test_did_peer_2_generation_with_services() {
        let keys: Vec<PurposedKey> = serde_json::from_str(
            r##"[{
                "purpose": "verification",
                "publicKeyMultibase": "z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"
            }]"##,
        )
        .unwrap();

        let services = vec![
            Service {
                id: String::from("#didcomm-1"),
                service_type: String::from("DIDCommMessaging"),
                service_endpoint: String::from("http://example.com/didcomm-1"),
                additional_properties: None,
            },
            Service {
                id: String::from("#didcomm-2"),
                service_type: String::from("DIDCommMessaging"),
                service_endpoint: String::from("http://example.com/didcomm-2"),
                additional_properties: None,
            },
        ];

        assert_eq!(
            &DIDPeerMethod::create_did_peer_2(&keys, &services).unwrap(),
            concat!(
                "did:peer:2",
                ".Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc",
                ".SeyJpZCI6IiNkaWRjb21tLTEiLCJzIjoiaHR0cDovL2V4YW1wbGUuY29tL2RpZGNvbW0tMSIsInQiOiJkbSJ9",
                ".SeyJpZCI6IiNkaWRjb21tLTIiLCJzIjoiaHR0cDovL2V4YW1wbGUuY29tL2RpZGNvbW0tMiIsInQiOiJkbSJ9"
            )
        );
    }

    #[test]
    fn test_did_peer_2_generation_should_err_on_key_associated_with_service_purpose() {
        let keys: Vec<PurposedKey> = serde_json::from_str(
            r##"[{
                "purpose": "service",
                "publicKeyMultibase": "z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"
            }]"##,
        )
        .unwrap();

        assert!(matches!(
            DIDPeerMethod::create_did_peer_2(&keys, &[]).unwrap_err(),
            DIDPeerMethodError::UnexpectedPurpose
        ));
    }

    #[test]
    fn test_did_peer_2_generation_should_err_on_empty_key_and_service_args() {
        assert!(matches!(
            DIDPeerMethod::create_did_peer_2(&[], &[]).unwrap_err(),
            DIDPeerMethodError::EmptyArguments
        ));
    }

    #[test]
    fn test_did_peer_3_generation() {
        let did = concat!(
            "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc.Vz6MkqRYqQi",
            "SgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF",
            "4ZnjhueYAFpEX6vg.SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2",
            "ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaW",
            "Rjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0",
        );

        assert_eq!(
            &DIDPeerMethod::create_did_peer_3(did).unwrap(),
            "did:peer:3zQmS19jtYDvGtKVrJhQnRFpBQAx3pJ9omx2HpNrcXFuRCz9"
        );
    }

    #[test]
    fn test_did_peer_3_generation_fails_on_non_did_peer_2_arg() {
        let dids = [
            "",
            "did:peer:0z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            "did:peer:1zQmbEB1EqP7PnNVaHiSpXhkatAA6kNyQK9mWkvrMx2eckgq",
        ];

        for did in dids {
            assert!(matches!(
                DIDPeerMethod::create_did_peer_3(did).unwrap_err(),
                DIDPeerMethodError::IllegalArgument
            ));
        }
    }
}
