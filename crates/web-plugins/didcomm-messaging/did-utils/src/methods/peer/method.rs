use multibase::Base::{Base58Btc, Base64Url};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{
    errors::DIDPeerMethodError,
    util::{abbreviate_service_for_did_peer_2, validate_input_document},
};

use crate::{
    crypto::{
        alg::decode_multikey,
        sha256_multihash, Algorithm, Ed25519KeyPair, PublicKeyFormat, {Generate, KeyMaterial},
    },
    didcore::{self, Document as DIDDocument, KeyFormat, Service, VerificationMethod},
    ldmodel::Context,
    methods::{errors::DIDResolutionError, peer::util, traits::DIDMethod, DidKey},
};

lazy_static::lazy_static!(
    pub static ref DID_PEER_0_REGEX: Regex = Regex::new("^did:peer:(0(z)([1-9a-km-zA-HJ-NP-Z]+))$").unwrap();
    pub static ref DID_PEER_2_REGEX: Regex = Regex::new("^did:peer:(2((\\.[AEVID](z)([1-9a-km-zA-HJ-NP-Z]+))+(\\.(S)[0-9a-zA-Z]*)*))$").unwrap();
);

const MULTICODEC_JSON: [u8; 2] = [0x80, 0x04];

#[derive(Default)]
pub struct DidPeer {
    /// Key format to consider during DID expansion into a DID document
    key_format: PublicKeyFormat,
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

impl DIDMethod for DidPeer {
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

impl DidPeer {
    /// Creates new instance of DidPeer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates new instance of DidPeer with given key format.
    pub fn with_format(key_format: PublicKeyFormat) -> Self {
        Self { key_format }
    }

    // ---------------------------------------------------------------------------
    // Generating did:peer addresses
    // ---------------------------------------------------------------------------

    /// Method 0: Generates did:peer address from ed25519 inception key without doc
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-0-inception-key-without-doc
    pub fn create_did_peer_0_from_ed25519_keypair(keypair: &Ed25519KeyPair) -> Result<String, DIDPeerMethodError> {
        let did_key = DidKey::from_ed25519_keypair(keypair)?;

        Ok(did_key.replace("did:key:", "did:peer:0"))
    }

    /// Method 0: Generates did:peer address from inception key without doc
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-0-inception-key-without-doc
    pub fn create_did_peer_0_from_raw_public_key(alg: Algorithm, bytes: &[u8]) -> Result<String, DIDPeerMethodError> {
        let did_key = DidKey::from_raw_public_key(alg, bytes)?;

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

        Ok(format!("did:peer:1{multihash}"))
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

        Ok(format!("did:peer:3{multihash}"))
    }

    /// Method 4: Generates did:peer address from DID document (embedding long form)
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-4-short-form-and-long-form
    pub fn create_did_peer_4_from_stored_variant(diddoc: &DIDDocument) -> Result<String, DIDPeerMethodError> {
        // Validate input documment
        validate_input_document(diddoc)?;

        // Encode document
        let json = json_canon::to_string(diddoc)?;
        let encoded = multibase::encode(Base58Btc, [&MULTICODEC_JSON, json.as_bytes()].concat());

        // Hashing
        let hash = sha256_multihash(encoded.as_bytes());

        Ok(format!("did:peer:4{hash}:{encoded}"))
    }

    /// Method 4: DID shortening for did:peer:4 addresses
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-4-short-form-and-long-form
    pub fn shorten_did_peer_4(did: &str) -> Result<String, DIDPeerMethodError> {
        let stripped = match did.strip_prefix("did:peer:4") {
            Some(stripped) => stripped,
            None => return Err(DIDPeerMethodError::IllegalArgument),
        };

        // Split hash and encoded segments
        let segments: Vec<_> = stripped.split(':').collect();
        if segments.len() != 2 || segments[1].is_empty() {
            return Err(DIDPeerMethodError::MalformedLongPeerDID);
        }

        let (hash, encoded) = (segments[0], segments[1]);

        // Verify hash
        if hash != sha256_multihash(encoded.as_bytes()) {
            return Err(DIDPeerMethodError::InvalidHash);
        }

        Ok(format!("did:peer:4{hash}"))
    }

    // ---------------------------------------------------------------------------
    // Expanding did:peer addresses
    // ---------------------------------------------------------------------------

    /// Expands `did:peer` address into DID document
    pub fn expand(&self, did: &str) -> Result<DIDDocument, DIDResolutionError> {
        if !did.starts_with("did:peer:") {
            return Err(DIDResolutionError::InvalidDid);
        }

        match did {
        s if s.starts_with("did:peer:0") => self.expand_did_peer_0(did).map_err(Into::into),
        s if s.starts_with("did:peer:2") => self.expand_did_peer_2(did).map_err(Into::into),
        s if s.starts_with("did:peer:4") => self.expand_did_peer_4(did).map_err(Into::into),
        _ => Err(DIDResolutionError::MethodNotSupported), 
    }
}


    /// Expands did:peer:0 address
    ///
    /// See https://identity.foundation/peer-did-method-spec/#method-0-inception-key-without-doc
    pub fn expand_did_peer_0(&self, did: &str) -> Result<DIDDocument, DIDPeerMethodError> {
        if !DID_PEER_0_REGEX.is_match(did) {
            return Err(DIDPeerMethodError::RegexMismatch);
        }

        // Decode multikey in did:peer
        let multikey = did.strip_prefix("did:peer:0").unwrap();
        let (alg, key) = decode_multikey(multikey).map_err(|_| DIDPeerMethodError::MalformedPeerDID)?;

        // Run algorithm for signature verification method expansion
        let signature_verification_method = self.derive_verification_method(did, multikey, alg, &key)?;

        // Build DID document
        let mut diddoc = DIDDocument {
            context: Context::SetOfString(vec![
                String::from("https://www.w3.org/ns/did/v1"),
                match self.key_format {
                    PublicKeyFormat::Multikey => String::from("https://w3id.org/security/multikey/v1"),
                    PublicKeyFormat::Jwk => String::from("https://w3id.org/security/suites/jws-2020/v1"),
                },
            ]),
            id: did.to_string(),
            controller: None,
            also_known_as: None,
            verification_method: Some(vec![signature_verification_method.clone()]),
            authentication: Some(vec![didcore::Authentication::Reference(
                signature_verification_method.id.clone(), //
            )]),
            assertion_method: Some(vec![didcore::AssertionMethod::Reference(
                signature_verification_method.id.clone(), //
            )]),
            capability_delegation: Some(vec![didcore::CapabilityDelegation::Reference(
                signature_verification_method.id.clone(), //
            )]),
            capability_invocation: Some(vec![didcore::CapabilityInvocation::Reference(
                signature_verification_method.id.clone(), //
            )]),
            key_agreement: None,
            service: None,
            additional_properties: None,
            proof: None,
        };

        // Derive X25519 key agreement if Ed25519
        if alg == Algorithm::Ed25519 {
            // Run algorithm for encryption verification method derivation
            let encryption_verification_method = self.derive_encryption_verification_method(did, &key)?;

            // Amend DID document accordingly
            let verification_method = diddoc.verification_method.as_mut().unwrap();
            verification_method.push(encryption_verification_method.clone());
            diddoc.key_agreement = Some(vec![didcore::KeyAgreement::Reference(
                encryption_verification_method.id.clone(), //
            )]);
        }

        // Output
        Ok(diddoc)
    }

    /// Derives verification method from multikey constituents
    fn derive_verification_method(&self, did: &str, multikey: &str, alg: Algorithm, key: &[u8]) -> Result<VerificationMethod, DIDPeerMethodError> {
        if let Some(required_length) = alg.public_key_length() {
            if required_length != key.len() {
                return Err(DIDResolutionError::InvalidPublicKeyLength.into());
            }
        }

        Ok(VerificationMethod {
            id: format!("#{multikey}"),
            key_type: String::from(match self.key_format {
                PublicKeyFormat::Multikey => "Multikey",
                PublicKeyFormat::Jwk => "JsonWebKey2020",
            }),
            controller: did.to_string(),
            public_key: Some(match self.key_format {
                PublicKeyFormat::Multikey => KeyFormat::Multibase(String::from(multikey)),
                PublicKeyFormat::Jwk => KeyFormat::Jwk(alg.build_jwk(key).map_err(|_| DIDResolutionError::InternalError)?),
            }),
            ..Default::default()
        })
    }

    /// Derives X25519 key agreement verification method indirectly from Ed25519 key
    fn derive_encryption_verification_method(&self, did: &str, key: &[u8]) -> Result<VerificationMethod, DIDPeerMethodError> {
        let key: [u8; 32] = key.try_into().map_err(|_| DIDResolutionError::InvalidPublicKeyLength)?;
        let ed25519_keypair = Ed25519KeyPair::from_public_key(&key).map_err(|_| DIDResolutionError::InternalError)?;
        let x25519_keypair = ed25519_keypair.get_x25519().map_err(|_| DIDResolutionError::InternalError)?;

        let alg = Algorithm::X25519;
        let enc_key = &x25519_keypair.public_key_bytes().unwrap()[..];
        let enc_multikey = multibase::encode(Base58Btc, [&alg.muticodec_prefix(), enc_key].concat());

        self.derive_verification_method(did, &enc_multikey, alg, enc_key)
    }

    /// Expands did:peer:2 address
    ///
    /// See https://identity.foundation/peer-did-method-spec/#resolving-a-didpeer2
    pub fn expand_did_peer_2(&self, did: &str) -> Result<DIDDocument, DIDPeerMethodError> {
        if !DID_PEER_2_REGEX.is_match(did) {
            return Err(DIDPeerMethodError::RegexMismatch);
        }

        // Compute did:peer:3 alias

        let alias = Self::create_did_peer_3(did)?;

        // Dissecting did address

        let chain = did.strip_prefix("did:peer:2.").unwrap();
        let chain: Vec<(Purpose, &str)> = chain
            .split('.')
            .map(|e| {
                (
                    Purpose::from_code(&e.chars().nth(0).unwrap()).expect("invalid purpose prefix bypasses regex check"),
                    &e[1..],
                )
            })
            .collect();

        // Define LD-JSON Context

        let context = Context::SetOfString(vec![
            String::from("https://www.w3.org/ns/did/v1"),
            match self.key_format {
                PublicKeyFormat::Multikey => String::from("https://w3id.org/security/multikey/v1"),
                PublicKeyFormat::Jwk => String::from("https://w3id.org/security/suites/jws-2020/v1"),
            },
        ]);

        // Initialize relationships

        let mut authentication = vec![];
        let mut assertion_method = vec![];
        let mut key_agreement = vec![];
        let mut capability_delegation = vec![];
        let mut capability_invocation = vec![];

        // Resolve verification methods

        let key_chain = chain.iter().filter(|(purpose, _)| purpose != &Purpose::Service);

        let mut methods: Vec<VerificationMethod> = vec![];

        for (method_current_id, (purpose, multikey)) in key_chain.enumerate() {
            let id = format!("#key-{}", method_current_id + 1);

            match purpose {
                Purpose::Assertion => assertion_method.push(didcore::AssertionMethod::Reference(id.clone())),
                Purpose::Encryption => key_agreement.push(didcore::KeyAgreement::Reference(id.clone())),
                Purpose::Verification => authentication.push(didcore::Authentication::Reference(id.clone())),
                Purpose::CapabilityDelegation => capability_delegation.push(didcore::CapabilityDelegation::Reference(id.clone())),
                Purpose::CapabilityInvocation => capability_invocation.push(didcore::CapabilityInvocation::Reference(id.clone())),
                Purpose::Service => unreachable!(),
            }

            let method = VerificationMethod {
                id,
                key_type: String::from(match self.key_format {
                    PublicKeyFormat::Multikey => "Multikey",
                    PublicKeyFormat::Jwk => "JsonWebKey2020",
                }),
                controller: did.to_string(),
                public_key: Some(match self.key_format {
                    PublicKeyFormat::Multikey => KeyFormat::Multibase(multikey.to_string()),
                    PublicKeyFormat::Jwk => {
                        let (alg, key) = decode_multikey(multikey).map_err(|_| DIDPeerMethodError::MalformedPeerDID)?;
                        KeyFormat::Jwk(alg.build_jwk(&key).map_err(|_| DIDResolutionError::InternalError)?)
                    }
                }),
                ..Default::default()
            };

            methods.push(method);
        }

        // Resolve services

        let service_chain = chain
            .iter()
            .filter_map(|(purpose, multikey)| (purpose == &Purpose::Service).then_some(multikey));

        let mut services: Vec<Service> = vec![];
        let mut service_next_id = 0;

        for encoded_service in service_chain {
            let decoded_service_bytes = Base64Url.decode(encoded_service).map_err(|_| DIDPeerMethodError::DIDParseError)?;
            let decoded_service = String::from_utf8(decoded_service_bytes).map_err(|_| DIDPeerMethodError::DIDParseError)?;

            // Reverse service abbreviation
            let mut service = util::reverse_abbreviate_service_for_did_peer_2(&decoded_service)?;

            if service.id.is_empty() {
                service.id = if service_next_id == 0 {
                    String::from("#service")
                } else {
                    format!("#service-{service_next_id}")
                };
                service_next_id += 1;
            }

            services.push(service);
        }

        // Build DIDDocument

        let diddoc = DIDDocument {
            context,
            id: did.to_string(),
            controller: None,
            also_known_as: Some(vec![alias]),
            verification_method: Some(methods),
            authentication: (!authentication.is_empty()).then_some(authentication),
            assertion_method: (!assertion_method.is_empty()).then_some(assertion_method),
            capability_delegation: (!capability_delegation.is_empty()).then_some(capability_delegation),
            capability_invocation: (!capability_invocation.is_empty()).then_some(capability_invocation),
            key_agreement: (!key_agreement.is_empty()).then_some(key_agreement),
            service: (!services.is_empty()).then_some(services),
            additional_properties: None,
            proof: None,
        };

        // Output

        Ok(diddoc)
    }

    /// Expands did:peer:4 address
    ///
    /// See https://identity.foundation/peer-did-method-spec/#resolving-a-did
    pub fn expand_did_peer_4(&self, did: &str) -> Result<DIDDocument, DIDPeerMethodError> {
        // Ensure long format by computing did:peer:4 short form alias
        // This also ensures that the hash is valid before shortening the did.
        let alias = Self::shorten_did_peer_4(did)?;

        // Extract encoded document
        let encoded_document = did.split(':').nth(3).ok_or(DIDPeerMethodError::DIDParseError)?;

        // Decode document
        let (base, decoded_bytes) = multibase::decode(encoded_document).map_err(|_| DIDPeerMethodError::DIDParseError)?;

        if Base58Btc != base || decoded_bytes.len() < 2 || decoded_bytes[..2] != MULTICODEC_JSON {
            return Err(DIDPeerMethodError::MalformedLongPeerDID);
        }
        // Deserialize the document
        let mut diddoc: DIDDocument = serde_json::from_slice(&decoded_bytes[2..]).map_err(DIDPeerMethodError::SerdeError)?;

        // Contextualize decoded document
        diddoc.id = did.to_string();

        if diddoc.also_known_as.is_none() {
            diddoc.also_known_as = Some(vec![alias]);
        }

        diddoc.verification_method = diddoc.verification_method.map(|arr| {
            arr.into_iter()
                .map(|vm| VerificationMethod {
                    controller: if !vm.controller.is_empty() { vm.controller } else { did.to_string() },
                    ..vm
                })
                .collect()
        });

        // Output DIDDocument
        Ok(diddoc)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    use crate::jwk::Jwk;

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

        let did = DidPeer::create_did_peer_0_from_ed25519_keypair(&keypair);
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
            let did = DidPeer::create_did_peer_0_from_raw_public_key(alg, &bytes);
            assert_eq!(did.unwrap(), expected);
        }
    }

    #[test]
    fn test_did_peer_1_generation_from_did_document() {
        let diddoc = _stored_variant_v0();
        let did = DidPeer::create_did_peer_1_from_stored_variant(&diddoc);
        assert_eq!(did.unwrap(), "did:peer:1zQmbEB1EqP7PnNVaHiSpXhkatAA6kNyQK9mWkvrMx2eckgq");
    }

    #[test]
    fn test_did_peer_1_generation_fails_from_did_document_with_id() {
        let diddoc = _invalid_stored_variant_v0();
        let did = DidPeer::create_did_peer_1_from_stored_variant(&diddoc);
        assert!(matches!(did.unwrap_err(), DIDPeerMethodError::InvalidStoredVariant));
    }

    #[test]
    fn test_did_peer_2_generation() {
        let keys = vec![
            PurposedKey {
                purpose: Purpose::Verification,
                public_key_multibase: String::from("z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"),
            },
            PurposedKey {
                purpose: Purpose::Encryption,
                public_key_multibase: String::from("z6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR"),
            },
        ];

        let did = DidPeer::create_did_peer_2(&keys, &[]).unwrap();
        assert_eq!(
            &did,
            "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR"
        );
    }

    #[test]
    fn test_did_peer_2_generation_with_service() {
        let keys = vec![PurposedKey {
            purpose: Purpose::Verification,
            public_key_multibase: String::from("z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"),
        }];

        let services = vec![Service {
            id: String::from("#didcomm"),
            service_type: String::from("DIDCommMessaging"),
            service_endpoint: String::from("http://example.com/didcomm"),
            additional_properties: None,
        }];

        assert_eq!(
            &DidPeer::create_did_peer_2(&keys, &services).unwrap(),
            concat!(
                "did:peer:2",
                ".Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc",
                ".SeyJpZCI6IiNkaWRjb21tIiwicyI6Imh0dHA6Ly9leGFtcGxlLmNvbS9kaWRjb21tIiwidCI6ImRtIn0"
            )
        );
    }

    #[test]
    fn test_did_peer_2_generation_with_services() {
        let keys = vec![PurposedKey {
            purpose: Purpose::Verification,
            public_key_multibase: String::from("z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"),
        }];

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
            &DidPeer::create_did_peer_2(&keys, &services).unwrap(),
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
        let keys = vec![PurposedKey {
            purpose: Purpose::Service,
            public_key_multibase: String::from("z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"),
        }];

        assert!(matches!(
            DidPeer::create_did_peer_2(&keys, &[]).unwrap_err(),
            DIDPeerMethodError::UnexpectedPurpose
        ));
    }

    #[test]
    fn test_did_peer_2_generation_should_err_on_empty_key_and_service_args() {
        assert!(matches!(
            DidPeer::create_did_peer_2(&[], &[]).unwrap_err(),
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
            &DidPeer::create_did_peer_3(did).unwrap(),
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
                DidPeer::create_did_peer_3(did).unwrap_err(),
                DIDPeerMethodError::IllegalArgument
            ));
        }
    }

    #[test]
    fn test_did_peer_4_generation() {
        let diddoc = _stored_variant_v0();
        assert_eq!(
            &DidPeer::create_did_peer_4_from_stored_variant(&diddoc).unwrap(),
            concat!(
                "did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZ:z2yS424R5nAoSu",
                "CezPTvBHybrvByZRD9g8L4oMe4ctq9UwPksVskxJFiars33RRyKz3z7RbwwQRAo9ByoXmBhg",
                "7UCMkvmSHBeXWF44tQJfLjiXieCtXgxASzPJ5UsgPLAWX2vdjNFfmiLVh1WLe3RdBPvQoMuM",
                "EiPLFGiKhbzX66dT21qDwZusRC4uDzQa7XpsLBS7rBjZZ9sLMRzjpG4rYpjgLUmUF2D1ixeW",
                "ZFMqy7fVfPUUGyt4N6R4aLAjMLgcJzAQKb1uFiBYe2ZCTmsjtazWkHypgJetLysv7AwasYDV",
                "4MMNPY5AbM4p3TGtdpJZaxaXzSKRZexuQ4tWsfGuHXEDiaABj5YtjbNjWh4f5M4sn7D9AAAS",
                "StG593VkLFaPxG4VnFR4tKPiWeN9AJXRWPQ2XRnsD7U3mCHpRSb2f1HT5KeSHTU8zNAn6vFc",
                "4fstgf2j71Uo8tngcUBkxdqkHKmpvZ1Fs27sWh7JvWAeiehsW3aBe4CbU4WGjzmusaKVb2HS",
                "7iY5hbYngYrpwcZ5Sse",
            )
        );
    }

    #[test]
    fn test_did_peer_4_generation_fails_from_did_document_with_id() {
        let diddoc = _invalid_stored_variant_v0();
        let did = DidPeer::create_did_peer_4_from_stored_variant(&diddoc);
        assert!(matches!(did.unwrap_err(), DIDPeerMethodError::InvalidStoredVariant));
    }

    #[test]
    fn test_did_peer_4_shortening() {
        let did = concat!(
            "did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZ:z2yS424R5nAoSu",
            "CezPTvBHybrvByZRD9g8L4oMe4ctq9UwPksVskxJFiars33RRyKz3z7RbwwQRAo9ByoXmBhg",
            "7UCMkvmSHBeXWF44tQJfLjiXieCtXgxASzPJ5UsgPLAWX2vdjNFfmiLVh1WLe3RdBPvQoMuM",
            "EiPLFGiKhbzX66dT21qDwZusRC4uDzQa7XpsLBS7rBjZZ9sLMRzjpG4rYpjgLUmUF2D1ixeW",
            "ZFMqy7fVfPUUGyt4N6R4aLAjMLgcJzAQKb1uFiBYe2ZCTmsjtazWkHypgJetLysv7AwasYDV",
            "4MMNPY5AbM4p3TGtdpJZaxaXzSKRZexuQ4tWsfGuHXEDiaABj5YtjbNjWh4f5M4sn7D9AAAS",
            "StG593VkLFaPxG4VnFR4tKPiWeN9AJXRWPQ2XRnsD7U3mCHpRSb2f1HT5KeSHTU8zNAn6vFc",
            "4fstgf2j71Uo8tngcUBkxdqkHKmpvZ1Fs27sWh7JvWAeiehsW3aBe4CbU4WGjzmusaKVb2HS",
            "7iY5hbYngYrpwcZ5Sse",
        );

        assert_eq!(
            &DidPeer::shorten_did_peer_4(did).unwrap(),
            "did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZ"
        );
    }

    #[test]
    fn test_did_peer_4_shortening_fails_on_non_did_peer_4_arg() {
        let dids = [
            "",
            "did:peer:0z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            "did:peer:1zQmbEB1EqP7PnNVaHiSpXhkatAA6kNyQK9mWkvrMx2eckgq",
            "did:peer:3zQmS19jtYDvGtKVrJhQnRFpBQAx3pJ9omx2HpNrcXFuRCz9",
        ];

        for did in dids {
            assert!(matches!(
                DidPeer::shorten_did_peer_4(did).unwrap_err(),
                DIDPeerMethodError::IllegalArgument
            ));
        }
    }

    #[test]
    fn test_did_peer_4_shortening_fails_on_malformed_long_peer_did() {
        let dids = [
            "did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZz2yS424R5nAoSu",
            "did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZ:z2yS424:R5nAoSu",
            "did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZ:",
        ];

        for did in dids {
            assert!(matches!(
                DidPeer::shorten_did_peer_4(did).unwrap_err(),
                DIDPeerMethodError::MalformedLongPeerDID
            ));
        }
    }

    #[test]
    fn test_did_peer_4_shortening_fails_on_invalid_hash_in_long_peer_did() {
        let valid_did = concat!(
            "did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZ:z2yS424R5nAoSu",
            "CezPTvBHybrvByZRD9g8L4oMe4ctq9UwPksVskxJFiars33RRyKz3z7RbwwQRAo9ByoXmBhg",
            "7UCMkvmSHBeXWF44tQJfLjiXieCtXgxASzPJ5UsgPLAWX2vdjNFfmiLVh1WLe3RdBPvQoMuM",
            "EiPLFGiKhbzX66dT21qDwZusRC4uDzQa7XpsLBS7rBjZZ9sLMRzjpG4rYpjgLUmUF2D1ixeW",
            "ZFMqy7fVfPUUGyt4N6R4aLAjMLgcJzAQKb1uFiBYe2ZCTmsjtazWkHypgJetLysv7AwasYDV",
            "4MMNPY5AbM4p3TGtdpJZaxaXzSKRZexuQ4tWsfGuHXEDiaABj5YtjbNjWh4f5M4sn7D9AAAS",
            "StG593VkLFaPxG4VnFR4tKPiWeN9AJXRWPQ2XRnsD7U3mCHpRSb2f1HT5KeSHTU8zNAn6vFc",
            "4fstgf2j71Uo8tngcUBkxdqkHKmpvZ1Fs27sWh7JvWAeiehsW3aBe4CbU4WGjzmusaKVb2HS",
            "7iY5hbYngYrpwcZ5Sse",
        );

        // Invalidate hash
        let mut did = valid_did.to_string();
        did.insert_str(20, "blurg");

        assert!(matches!(DidPeer::shorten_did_peer_4(&did).unwrap_err(), DIDPeerMethodError::InvalidHash));

        // Invalidate hash by tampering with encoded document
        let did = format!("{valid_did}blurg");

        assert!(matches!(DidPeer::shorten_did_peer_4(&did).unwrap_err(), DIDPeerMethodError::InvalidHash));
    }

    #[test]
    fn test_expand_fails_on_non_did_peer() {
        let did_method = DidPeer::default();

        let did = "did:key:z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F";
        assert!(matches!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidDid));
    }

    #[test]
    fn test_expand_fails_on_unsupported_did_peer() {
        let did_method = DidPeer::default();

        let did = "did:peer:1zQmbEB1EqP7PnNVaHiSpXhkatAA6kNyQK9mWkvrMx2eckgq";
        assert!(matches!(
            did_method.expand(did).unwrap_err(),
            DIDResolutionError::MethodNotSupported
        ));
    }

    #[test]
    fn test_expand_did_peer_0_v1() {
        let did_method = DidPeer::default();

        let did = "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let expected: Value = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/multikey/v1"
                ],
                "id": "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "verificationMethod": [
                    {
                        "id": "#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "type": "Multikey",
                        "controller": "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
                    },
                    {
                        "id": "#z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p",
                        "type": "Multikey",
                        "controller": "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "publicKeyMultibase": "z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p"
                    }
                ],
                "authentication": ["#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "assertionMethod": ["#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityDelegation": ["#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityInvocation": ["#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "keyAgreement": ["#z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p"]
            }"##,
        )
        .unwrap();

        let diddoc = did_method.expand(did).unwrap();

        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[test]
    fn test_expand_did_peer_0_v2() {
        let did_method = DidPeer::default();

        let did = "did:peer:0z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F";
        let expected: Value = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/multikey/v1"
                ],
                "id": "did:peer:0z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F",
                "verificationMethod": [
                    {
                        "id": "#z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F",
                        "type": "Multikey",
                        "controller": "did:peer:0z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F",
                        "publicKeyMultibase": "z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F"
                    }
                ],
                "authentication": ["#z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F"],
                "assertionMethod": ["#z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F"],
                "capabilityDelegation": ["#z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F"],
                "capabilityInvocation": ["#z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F"]
            }"##,
        )
        .unwrap();

        let diddoc = did_method.expand(did).unwrap();

        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[test]
    fn test_expand_did_peer_0_jwk_format() {
        let did_method = DidPeer {
            key_format: PublicKeyFormat::Jwk,
        };

        let did = "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let expected: Value = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/jws-2020/v1"
                ],
                "id": "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "verificationMethod": [
                    {
                        "id": "#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": "Lm_M42cB3HkUiODQsXRcweM6TByfzEHGO9ND274JcOY"
                        }
                    },
                    {
                        "id": "#z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "X25519",
                            "x": "bl_3kgKpz9jgsg350CNuHa_kQL3B60Gi-98WmdQW2h8"
                        }
                    }
                ],
                "authentication": ["#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "assertionMethod": ["#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityDelegation": ["#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityInvocation": ["#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "keyAgreement": ["#z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p"]
            }"##,
        )
        .unwrap();

        let diddoc = did_method.expand(did).unwrap();

        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[test]
    fn test_expand_did_peer_0_fails_for_regex_mismatch() {
        let did_method = DidPeer::default();

        let dids = [
            // Must be '0z' not '0Z'
            "did:peer:0Z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            // '#' is not a valid Base58 character
            "did:peer:0z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDoo###",
        ];

        for did in dids {
            assert!(matches!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidDid));
        }
    }

    #[test]
    fn test_expand_did_peer_0_fails_on_malformed_dids() {
        let did_method = DidPeer::default();

        let dids = ["did:peer:0z6", "did:peer:0z7MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWpd"];

        for did in dids {
            assert!(matches!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidDid));
        }
    }

    #[test]
    fn test_expand_did_peer_0_fails_on_too_long_did() {
        let did_method = DidPeer::default();
        let did = "did:peer:0zQebt6zPwbE4Vw5GFAjjARHrNXFALofERVv4q6Z4db8cnDRQm";
        assert!(matches!(
            did_method.expand(did).unwrap_err(),
            DIDResolutionError::InvalidPublicKeyLength
        ));
    }

    #[test]
    fn test_expand_did_peer_2() {
        let did_method = DidPeer::default();

        let did = concat!(
            "did:peer:2",
            ".Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc",
            ".Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR",
            ".SeyJpZCI6IiNkaWRjb21tIiwicyI6Imh0dHA6Ly9leGFtcGxlLmNvbS8xMjMiLCJ0IjoiZG0ifQ",
            ".SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20vYWJjIiwidCI6ImRtIn0",
            ".SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20veHl6IiwidCI6ImRtIn0",
        );

        let expected: Value = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/multikey/v1"
                ],
                "id": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJpZCI6IiNkaWRjb21tIiwicyI6Imh0dHA6Ly9leGFtcGxlLmNvbS8xMjMiLCJ0IjoiZG0ifQ.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20vYWJjIiwidCI6ImRtIn0.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20veHl6IiwidCI6ImRtIn0",
                "alsoKnownAs": ["did:peer:3zQmR9j6bEaydJuXDfzYaW4f3EEPQFxz2Zy1iPZuchgeF63h"],
                "verificationMethod": [
                    {
                        "id": "#key-1",
                        "type": "Multikey",
                        "controller": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJpZCI6IiNkaWRjb21tIiwicyI6Imh0dHA6Ly9leGFtcGxlLmNvbS8xMjMiLCJ0IjoiZG0ifQ.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20vYWJjIiwidCI6ImRtIn0.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20veHl6IiwidCI6ImRtIn0",
                        "publicKeyMultibase": "z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"
                    },
                    {
                        "id": "#key-2",
                        "type": "Multikey",
                        "controller": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJpZCI6IiNkaWRjb21tIiwicyI6Imh0dHA6Ly9leGFtcGxlLmNvbS8xMjMiLCJ0IjoiZG0ifQ.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20vYWJjIiwidCI6ImRtIn0.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20veHl6IiwidCI6ImRtIn0",
                        "publicKeyMultibase": "z6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR"
                    }
                ],
                "authentication": ["#key-1"],
                "keyAgreement": ["#key-2"],
                "service": [
                    {
                        "id": "#didcomm",
                        "type": "DIDCommMessaging",
                        "serviceEndpoint": "http://example.com/123"
                    },
                    {
                        "id": "#service",
                        "type": "DIDCommMessaging",
                        "serviceEndpoint": "http://example.com/abc"
                    },
                    {
                        "id": "#service-1",
                        "type": "DIDCommMessaging",
                        "serviceEndpoint": "http://example.com/xyz"
                    }
                ]
            }"##,
        )
        .unwrap();

        let diddoc = did_method.expand(did).unwrap();
        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[test]
    fn test_expand_did_peer_2_jwk_format() {
        let did_method = DidPeer {
            key_format: PublicKeyFormat::Jwk,
        };

        let did = concat!(
            "did:peer:2",
            ".Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc",
            ".Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR",
            ".SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20vYWJjIiwidCI6ImRtIn0",
        );

        let expected: Value = serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/jws-2020/v1"
                ],
                "id": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20vYWJjIiwidCI6ImRtIn0",
                "alsoKnownAs": ["did:peer:3zQmWdmF5Lgads1v6qeV9x6PJEWrfUaKQ5D8tf7up9a5xwDi"],
                "verificationMethod": [
                    {
                        "id": "#key-1",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20vYWJjIiwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "x": "RCzl6iYBsyD4aK8Yzmo8r_6eBriu0XmnDj64xOQ3d6M"
                        }
                    },
                    {
                        "id": "#key-2",
                        "type": "JsonWebKey2020",
                        "controller": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.SeyJpZCI6IiIsInMiOiJodHRwOi8vZXhhbXBsZS5jb20vYWJjIiwidCI6ImRtIn0",
                        "publicKeyJwk": {
                            "kty": "OKP",
                            "crv": "X25519",
                            "x": "Qk1FMFvAv5Ihlgjm_SJIqNRU3kqhb_RWQZrPUh3mNWg"
                        }
                    }
                ],
                "authentication": ["#key-1"],
                "keyAgreement": ["#key-2"],
                "service": [{
                    "id": "#service",
                    "type": "DIDCommMessaging",
                    "serviceEndpoint": "http://example.com/abc"
                }]
            }"##,
        )
        .unwrap();

        let diddoc = did_method.expand(did).unwrap();
        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[test]
    fn test_expand_did_peer_2_fails_on_malformed_encoded_service() {
        let did_method = DidPeer::default();
        let did = concat!(
            "did:peer:2",
            ".Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc",
            // {"s":"http://example.com/xyz","t":"dm" (missing closing brace)
            ".SeyJzIjoiaHR0cDovL2V4YW1wbGUuY29tL3h5eiIsInQiOiJkbSI",
        );

        assert!(matches!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidDid));
    }

    #[test]
    fn test_expand_did_peer_4() {
        let did_method = DidPeer::new();

        let did = concat!(
            "did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZ:z2yS424R5nAoSu",
            "CezPTvBHybrvByZRD9g8L4oMe4ctq9UwPksVskxJFiars33RRyKz3z7RbwwQRAo9ByoXmBhg",
            "7UCMkvmSHBeXWF44tQJfLjiXieCtXgxASzPJ5UsgPLAWX2vdjNFfmiLVh1WLe3RdBPvQoMuM",
            "EiPLFGiKhbzX66dT21qDwZusRC4uDzQa7XpsLBS7rBjZZ9sLMRzjpG4rYpjgLUmUF2D1ixeW",
            "ZFMqy7fVfPUUGyt4N6R4aLAjMLgcJzAQKb1uFiBYe2ZCTmsjtazWkHypgJetLysv7AwasYDV",
            "4MMNPY5AbM4p3TGtdpJZaxaXzSKRZexuQ4tWsfGuHXEDiaABj5YtjbNjWh4f5M4sn7D9AAAS",
            "StG593VkLFaPxG4VnFR4tKPiWeN9AJXRWPQ2XRnsD7U3mCHpRSb2f1HT5KeSHTU8zNAn6vFc",
            "4fstgf2j71Uo8tngcUBkxdqkHKmpvZ1Fs27sWh7JvWAeiehsW3aBe4CbU4WGjzmusaKVb2HS",
            "7iY5hbYngYrpwcZ5Sse",
        );

        let expected = DIDDocument {
            id: did.to_string(),
            also_known_as: Some(vec![String::from("did:peer:4zQmePYVawceZsPSxpLRp54z4Q5DCZXeyyGKwoDMc2NqgZXZ")]),
            .._stored_variant_v0()
        };

        let diddoc = did_method.expand(did).unwrap();
        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    fn _stored_variant_v0() -> DIDDocument {
        serde_json::from_str(
            r##"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/ed25519-2020/v1"
                ],
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
        .unwrap()
    }

    fn _invalid_stored_variant_v0() -> DIDDocument {
        serde_json::from_str(
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
        .unwrap()
    }
}
