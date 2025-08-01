use multibase::Base::Base58Btc;

use crate::{
    crypto::{
        alg::decode_multikey,
        Algorithm, Ed25519KeyPair, Error as CryptoError, PublicKeyFormat, {Generate, KeyMaterial},
    },
    didcore::{self, Document as DIDDocument, KeyFormat, VerificationMethod},
    ldmodel::Context,
    methods::{errors::DIDResolutionError, traits::DIDMethod},
};

/// A struct for resolving DID Key documents.
#[derive(Default)]
pub struct DidKey {
    /// Key format to consider during DID expansion into a DID document
    key_format: PublicKeyFormat,

    /// Derive key agreement on expanding did:key address
    enable_encryption_key_derivation: bool,
}

impl DIDMethod for DidKey {
    fn name() -> String {
        "did:key".to_string()
    }
}

impl DidKey {
    /// Creates a new DidKey resolver instance.
    pub fn new() -> Self {
        Self::new_full(false, PublicKeyFormat::default())
    }

    /// Creates a new DidKey resolver with optional encryption key derivation and a specific key format.
    pub fn new_full(enable_encryption_key_derivation: bool, key_format: PublicKeyFormat) -> Self {
        Self {
            enable_encryption_key_derivation,
            key_format,
        }
    }

    /// Generates a new DID key from an Ed25519 key pair.
    ///
    /// This function creates a new Ed25519 key pair and converts it into a DID key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use did_utils::methods::DidKey;
    ///
    /// # fn example() -> Result<(), did_utils::crypto::Error> {
    /// let did_key = DidKey::generate()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate() -> Result<String, CryptoError> {
        let keypair = Ed25519KeyPair::new()?;
        Self::from_ed25519_keypair(&keypair)
    }

    /// Converts an Ed25519 key pair into a DID key.
    ///
    /// This function takes an existing Ed25519 key pair and returns the corresponding DID key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use did_utils::crypto::{Ed25519KeyPair, Generate};
    /// use did_utils::methods::DidKey;
    ///
    /// # fn example() -> Result<(), did_utils::crypto::Error> {
    /// let keypair = Ed25519KeyPair::new()?;
    /// let did_key = DidKey::from_ed25519_keypair(&keypair)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_ed25519_keypair(keypair: &Ed25519KeyPair) -> Result<String, CryptoError> {
        let multibase_value = multibase::encode(
            Base58Btc,
            [&Algorithm::Ed25519.muticodec_prefix(), keypair.public_key_bytes()?.as_slice()].concat(),
        );

        Ok(format!("did:key:{multibase_value}"))
    }

    /// Converts a raw public key into a DID key.
    ///
    /// This function takes a raw public key and an algorithm type, and returns the corresponding DID key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use did_utils::methods::DidKey;
    /// use did_utils::methods::Algorithm;
    ///
    /// # fn example() -> Result<(), did_utils::crypto::Error> {
    /// let bytes = [0u8; 32];
    /// let did_key = DidKey::from_raw_public_key(Algorithm::Ed25519, &bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function returns an error if the length of the raw public key does not match the algorithm's expected length.
    pub fn from_raw_public_key(alg: Algorithm, bytes: &[u8]) -> Result<String, CryptoError> {
        if let Some(required_length) = alg.public_key_length() {
            if required_length != bytes.len() {
                return Err(CryptoError::InvalidKeyLength);
            }
        }

        let multibase_value = multibase::encode(Base58Btc, [&alg.muticodec_prefix(), bytes].concat());

        Ok(format!("did:key:{multibase_value}"))
    }

    /// Expands `did:key` address into DID document
    ///
    /// See [Create a did key](https://w3c-ccg.github.io/did-method-key/#create)
    pub fn expand(&self, did: &str) -> Result<DIDDocument, DIDResolutionError> {
        if !did.starts_with("did:key:") {
            return Err(DIDResolutionError::InvalidDid);
        }

        // See https://w3c-ccg.github.io/did-method-key/#format
        let multibase_value = did.strip_prefix("did:key:").unwrap();
        let (alg, raw_public_key_bytes) = decode_multikey(multibase_value).map_err(|_| DIDResolutionError::InvalidDid)?;

        // Run algorithm for signature verification method expansion
        let signature_verification_method = self.derive_signature_verification_method(alg, multibase_value, &raw_public_key_bytes)?;

        // Build DID document
        let mut diddoc = DIDDocument {
            context: Context::SetOfString(self.guess_context_property(&alg)),
            id: did.to_string(),
            controller: None,
            also_known_as: None,
            verification_method: Some(vec![signature_verification_method.clone()]),
            authentication: Some(vec![didcore::VerificationMethodType::Reference(
                signature_verification_method.id.clone(), //
            )]),
            assertion_method: Some(vec![didcore::VerificationMethodType::Reference(
                signature_verification_method.id.clone(), //
            )]),
            capability_delegation: Some(vec![didcore::VerificationMethodType::Reference(
                signature_verification_method.id.clone(), //
            )]),
            capability_invocation: Some(vec![didcore::VerificationMethodType::Reference(
                signature_verification_method.id.clone(), //
            )]),
            key_agreement: None,
            service: None,
            additional_properties: None,
            proof: None,
        };

        if self.enable_encryption_key_derivation {
            // Run algorithm for encryption verification method derivation if opted in
            let encryption_verification_method = self.derive_encryption_verification_method(alg, multibase_value, &raw_public_key_bytes)?;

            // Amend DID document accordingly
            let verification_method = diddoc.verification_method.as_mut().unwrap();
            verification_method.push(encryption_verification_method.clone());
            diddoc.key_agreement = Some(vec![didcore::VerificationMethodType::Reference(
                encryption_verification_method.id.clone(), //
            )]);
        }

        Ok(diddoc)
    }

    fn guess_context_property(&self, alg: &Algorithm) -> Vec<String> {
        let mut context = vec!["https://www.w3.org/ns/did/v1"];

        match self.key_format {
            PublicKeyFormat::Multikey => match alg {
                Algorithm::Ed25519 => {
                    context.push("https://w3id.org/security/suites/ed25519-2020/v1");
                    if self.enable_encryption_key_derivation {
                        context.push("https://w3id.org/security/suites/x25519-2020/v1");
                    }
                }
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
        alg: Algorithm,
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
                PublicKeyFormat::Jwk => KeyFormat::Jwk(Box::new(
                    alg.build_jwk(raw_public_key_bytes) //
                        .map_err(|_| DIDResolutionError::InternalError)?,
                )),
            }),
            ..Default::default()
        })
    }

    // See https://w3c-ccg.github.io/did-method-key/#encryption-method-creation-algorithm
    fn derive_encryption_verification_method(
        &self,
        alg: Algorithm,
        multibase_value: &str,
        raw_public_key_bytes: &[u8],
    ) -> Result<VerificationMethod, DIDResolutionError> {
        if alg != Algorithm::Ed25519 {
            return Err(DIDResolutionError::InternalError);
        }

        let raw_public_key_bytes: [u8; 32] = raw_public_key_bytes.try_into().map_err(|_| DIDResolutionError::InvalidPublicKeyLength)?;
        let ed25519_keypair = Ed25519KeyPair::from_public_key(&raw_public_key_bytes).map_err(|_| DIDResolutionError::InternalError)?;
        let x25519_keypair = ed25519_keypair.get_x25519().map_err(|_| DIDResolutionError::InternalError)?;

        let alg = Algorithm::X25519;
        let encryption_raw_public_key_bytes = &x25519_keypair.public_key_bytes().unwrap()[..];
        let encryption_multibase_value = multibase::encode(Base58Btc, [&alg.muticodec_prefix(), encryption_raw_public_key_bytes].concat());

        Ok(VerificationMethod {
            id: format!("did:key:{multibase_value}#{encryption_multibase_value}"),
            key_type: String::from(match self.key_format {
                PublicKeyFormat::Multikey => "X25519KeyAgreementKey2020",
                PublicKeyFormat::Jwk => "JsonWebKey2020",
            }),
            controller: format!("did:key:{multibase_value}"),
            public_key: Some(match self.key_format {
                PublicKeyFormat::Multikey => KeyFormat::Multibase(encryption_multibase_value),
                PublicKeyFormat::Jwk => KeyFormat::Jwk(Box::new(
                    alg.build_jwk(encryption_raw_public_key_bytes)
                        .map_err(|_| DIDResolutionError::InternalError)?,
                )),
            }),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwk::Jwk;
    use serde_json::Value;

    #[test]
    fn test_did_key_generation() {
        let did = DidKey::generate();
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

        let did = DidKey::from_ed25519_keypair(&keypair);
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
            let did = DidKey::from_raw_public_key(alg, &bytes);
            assert_eq!(did.unwrap(), expected);
        }
    }

    #[test]
    fn test_did_key_expansion_multikey() {
        let did_method = DidKey::new();

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
        let did_method = DidKey::new_full(false, PublicKeyFormat::Jwk);

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
    fn test_did_key_expansion_multikey_with_encryption_derivation() {
        let did_method = DidKey::new_full(true, PublicKeyFormat::default());

        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let expected: Value = serde_json::from_str(
            r#"{
                "@context": [
                    "https://www.w3.org/ns/did/v1",
                    "https://w3id.org/security/suites/ed25519-2020/v1",
                    "https://w3id.org/security/suites/x25519-2020/v1"
                ],
                "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "verificationMethod": [
                    {
                        "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "type": "Ed25519VerificationKey2020",
                        "controller": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
                    },
                    {
                        "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p",
                        "type": "X25519KeyAgreementKey2020",
                        "controller": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                        "publicKeyMultibase": "z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p"
                    }
                ],
                "authentication": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "assertionMethod": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityDelegation": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "capabilityInvocation": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"],
                "keyAgreement": ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p"]
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
        let did_method = DidKey::new();

        let did = "did:key:Z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        assert_eq!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidDid);

        let did = "did:key:z6MkhaXgBZDvotDkL5257####tiGiC2QtKLGpbnnEGta2doK";
        assert_eq!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidDid);

        let did = "did:key:zQebt6zPwbE4Vw5GFAjjARHrNXFALofERVv4q6Z4db8cnDRQm";
        assert_eq!(did_method.expand(did).unwrap_err(), DIDResolutionError::InvalidPublicKeyLength);
    }
}
