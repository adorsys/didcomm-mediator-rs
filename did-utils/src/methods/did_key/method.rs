use multibase::Base::Base58Btc;

use crate::{
    crypto::{
        ed25519::Ed25519KeyPair,
        traits::{Error as CryptoError, Generate, KeyMaterial},
    },
    didcore::{self, Document as DIDDocument, KeyFormat, VerificationMethod},
    ldmodel::Context,
    methods::{errors::DIDResolutionError, traits::DIDMethod},
};

use super::alg::Algorithm;

#[derive(Default)]
pub enum PublicKeyFormat {
    #[default]
    Multikey,
    Jwk,
}

#[derive(Default)]
pub struct DIDKeyMethod {
    /// Key format to consider during DID expansion into a DID document
    pub key_format: PublicKeyFormat,

    /// Derive key agreement on expanding did:key address
    pub enable_encryption_key_derivation: bool,
}

impl DIDMethod for DIDKeyMethod {
    fn name() -> String {
        "did:key".to_string()
    }
}

impl DIDKeyMethod {
    /// Generates did:key address ex nihilo, off self-generated Ed25519 key pair
    pub fn generate() -> Result<String, CryptoError> {
        let keypair = Ed25519KeyPair::new()?;
        Self::from_ed25519_keypair(&keypair)
    }

    /// Computes did:key address corresponding to Ed25519 key pair
    pub fn from_ed25519_keypair(keypair: &Ed25519KeyPair) -> Result<String, CryptoError> {
        let multibase_value = multibase::encode(
            Base58Btc,
            [&Algorithm::Ed25519.muticodec_prefix(), keypair.public_key_bytes()?.as_slice()].concat(),
        );

        Ok(format!("did:key:{}", multibase_value))
    }

    /// Computes did:key address corresponding to raw public key bytes
    pub fn from_raw_public_key(alg: Algorithm, bytes: &[u8]) -> Result<String, CryptoError> {
        if let Some(required_length) = alg.public_key_length() {
            if required_length != bytes.len() {
                return Err(CryptoError::InvalidKeyLength);
            }
        }

        let multibase_value = multibase::encode(Base58Btc, [&alg.muticodec_prefix(), bytes].concat());

        Ok(format!("did:key:{}", multibase_value))
    }

    /// Expands did:key address into DID document
    ///
    /// See https://w3c-ccg.github.io/did-method-key/#create
    pub fn expand(&self, did: &str) -> Result<DIDDocument, DIDResolutionError> {
        if !did.starts_with("did:key:") {
            return Err(DIDResolutionError::InvalidDID);
        }

        let multibase_value = did.strip_prefix("did:key:").unwrap();
        let (base, multicodec) = multibase::decode(multibase_value).map_err(|_| DIDResolutionError::InvalidDID)?;

        // Validate decoded multibase value
        if base != Base58Btc || multicodec.len() < 2 {
            return Err(DIDResolutionError::InvalidDID);
        }

        // Partition multicodec value
        let multicodec_prefix: &[u8; 2] = &multicodec[..2].try_into().unwrap();
        let raw_public_key_bytes = &multicodec[2..];

        // Derive algorithm from multicodec prefix
        let alg = Algorithm::from_muticodec_prefix(multicodec_prefix).ok_or(DIDResolutionError::InvalidDID)?;

        // Run algorithm for signature verification method expansion
        let signature_verification_method = self.derive_signature_verification_method(&alg, multibase_value, raw_public_key_bytes)?;

        // Build DID document
        let diddoc = DIDDocument {
            context: Context::SetOfString(self.guess_context_property(&alg)),
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

        Ok(diddoc)
    }

    fn guess_context_property(&self, alg: &Algorithm) -> Vec<String> {
        let mut context = vec!["https://www.w3.org/ns/did/v1"];

        match self.key_format {
            PublicKeyFormat::Multikey => match alg {
                Algorithm::Ed25519 => context.push("https://w3id.org/security/suites/ed25519-2020/v1"),
                Algorithm::X25519 => context.push("https://w3id.org/security/suites/x25519-2020/v1"),
                _ => (),
            },
            PublicKeyFormat::Jwk => context.push("https://w3id.org/security/suites/jws-2020/v1"),
        }

        context.iter().map(|x| x.to_string()).collect()
    }

    // See https://w3c-ccg.github.io/did-method-key/#signature-method-creation-algorithm
    fn derive_signature_verification_method(
        &self,
        alg: &Algorithm,
        multibase_value: &str,
        raw_public_key_bytes: &[u8],
    ) -> Result<VerificationMethod, DIDResolutionError> {
        if let Some(required_length) = alg.public_key_length() {
            if required_length != raw_public_key_bytes.len() {
                return Err(DIDResolutionError::InvalidPublicKeyLength);
            }
        }

        Ok(VerificationMethod {
            id: format!("did:key:{multibase_value}#{multibase_value}"),
            key_type: String::from(match self.key_format {
                PublicKeyFormat::Multikey => match alg {
                    Algorithm::Ed25519 => "Ed25519VerificationKey2020",
                    Algorithm::X25519 => "X25519KeyAgreementKey2020",
                    _ => "Multikey",
                },
                PublicKeyFormat::Jwk => "JsonWebKey2020",
            }),
            controller: format!("did:key:{multibase_value}"),
            public_key: Some(match self.key_format {
                PublicKeyFormat::Multikey => KeyFormat::Multibase(String::from(multibase_value)),
                PublicKeyFormat::Jwk => KeyFormat::Jwk(alg.build_jwk(raw_public_key_bytes)),
            }),
            ..VerificationMethod::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use crate::didcore::Jwk;

    #[test]
    fn test_did_key_generation() {
        let did = DIDKeyMethod::generate();
        assert!(did.unwrap().starts_with("did:key:z6Mk"));
    }

    #[test]
    fn test_did_key_generation_from_given_jwk() {
        let jwk: Jwk = serde_json::from_str(
            r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "O2onvM62pC1io6jQKm8Nc2UyFXcd4kOmOsBIoYtZ2ik"
            }"#,
        )
        .unwrap();
        let keypair: Ed25519KeyPair = jwk.try_into().unwrap();

        let did = DIDKeyMethod::from_ed25519_keypair(&keypair);
        assert_eq!(did.unwrap(), "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp");
    }

    #[test]
    fn test_did_key_generation_from_given_raw_public_key_bytes() {
        let entries = [
            (
                Algorithm::Ed25519,
                hex::decode("3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29").unwrap(),
                "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            ),
            (
                Algorithm::X25519,
                hex::decode("2fe57da347cd62431528daac5fbb290730fff684afc4cfc2ed90995f58cb3b74").unwrap(),
                "did:key:z6LSeu9HkTHSfLLeUs2nnzUSNedgDUevfNQgQjQC23ZCit6F",
            ),
            (
                Algorithm::Secp256k1,
                hex::decode("03874c15c7fda20e539c6e5ba573c139884c351188799f5458b4b41f7924f235cd").unwrap(),
                "did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme",
            ),
            (
                Algorithm::P521,
                hex::decode("020125073ccca272143441b1d9f687cdc7f978cbb96e9dc9f97de28ba373a92769d26d9a02ee67dfa258f9bb2eece8a48a5c59a7356c46278d883ab8d9e3baaac2ac92").unwrap(),
                "did:key:z2J9gaYxrKVpdoG9A4gRnmpnRCcxU6agDtFVVBVdn1JedouoZN7SzcyREXXzWgt3gGiwpoHq7K68X4m32D8HgzG8wv3sY5j7",
            )
        ];

        for entry in entries {
            let (alg, bytes, expected) = entry;
            let did = DIDKeyMethod::from_raw_public_key(alg, &bytes);
            assert_eq!(did.unwrap(), expected);
        }
    }

    #[test]
    fn test_did_key_expansion_multikey() {
        let did_method = DIDKeyMethod::default();

        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let expected: Value = serde_json::from_str(
            r#"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/ed25519-2020/v1"
                ],
                "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "verificationMethod": [{
                    "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                    "type": "Ed25519VerificationKey2020",
                    "controller": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                    "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
                }],
                "authentication": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "assertionMethod": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityDelegation": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityInvocation": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
            }"#,
        )
        .unwrap();

        let diddoc = did_method.expand(did).unwrap();

        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[test]
    fn test_did_key_expansion_jsonwebkey() {
        let did_method = DIDKeyMethod {
            key_format: PublicKeyFormat::Jwk,
            ..Default::default()
        };

        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let expected: Value = serde_json::from_str(
            r#"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/jws-2020/v1"
                ],
                "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "verificationMethod": [{
                    "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                    "type": "JsonWebKey2020",
                    "controller": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                    "publicKeyJwk": {
                        "kty": "OKP",
                        "crv": "Ed25519",
                        "x": "Lm_M42cB3HkUiODQsXRcweM6TByfzEHGO9ND274JcOY"
                    }
                }],
                "authentication": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "assertionMethod": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityDelegation": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityInvocation": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
            }"#,
        )
        .unwrap();

        let diddoc = did_method.expand(did).unwrap();

        assert_eq!(
            json_canon::to_string(&diddoc).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[test]
    fn test_did_key_expansion_fails_as_expected() {
        let did_method = DIDKeyMethod::default();

        let did = "did:key:Z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        assert_eq!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidDID);

        let did = "did:key:z6MkhaXgBZDvotDkL5257####tiGiC2QtKLGpbnnEGta2doK";
        assert_eq!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidDID);

        let did = "did:key:zQebt6zPwbE4Vw5GFAjjARHrNXFALofERVv4q6Z4db8cnDRQm";
        assert_eq!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidPublicKeyLength);
    }
}
