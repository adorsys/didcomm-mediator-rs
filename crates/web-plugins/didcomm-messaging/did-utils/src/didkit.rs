use serde_json::Value;

use crate::{
    didcore::{Document, Service, VerificationMethod, VerificationMethodType},
    ldmodel::Context,
};

impl VerificationMethod {
    /// Creates a new `VerificationMethod` instance with only the required members.
    pub fn new(id: String, key_type: String, controller: String) -> Self {
        Self {
            id,
            key_type,
            controller,
            revoked: None,
            public_key: None,
            private_key: None,
            additional_properties: None,
        }
    }
}

impl Service {
    /// Creates a new `Service` instance with only the required members.
    pub fn new(id: String, service_type: String, service_endpoint: Value) -> Self {
        Self {
            id,
            service_type,
            service_endpoint,
            additional_properties: None,
        }
    }
}

impl Document {
    /// Creates a new `Document` instance with frequent members.
    pub fn new(context: Context, id: String) -> Self {
        Self {
            id,
            context,
            also_known_as: None,
            controller: None,
            authentication: Some(vec![]),
            assertion_method: Some(vec![]),
            capability_delegation: None,
            capability_invocation: None,
            key_agreement: Some(vec![]),
            verification_method: Some(vec![]),
            service: Some(vec![]),
            proof: None,
            additional_properties: None,
        }
    }

    /// Creates a new `Document` instance with frequent members.
    pub fn new_full(
        context: Context,
        id: String,
        authentication: Option<Vec<VerificationMethodType>>,
        assertion_method: Option<Vec<VerificationMethodType>>,
        key_agreement: Option<Vec<VerificationMethodType>>,
        verification_method: Option<Vec<VerificationMethod>>,
        service: Option<Vec<Service>>,
    ) -> Self {
        Self {
            id,
            context,
            also_known_as: None,
            controller: None,
            authentication,
            assertion_method,
            capability_delegation: None,
            capability_invocation: None,
            key_agreement,
            verification_method,
            service,
            proof: None,
            additional_properties: None,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::crypto::{
        Ed25519KeyPair, {Generate, KeyMaterial},
    };
    use crate::didcore::KeyFormat::Multibase;
    use multibase::Base::Base58Btc;

    #[test]
    fn test_document_new() {
        let context = Context::SingleString("https://www.w3.org/ns/did/v1".to_owned());
        let id = "did:example:123456789abcdefghi".to_string();
        let document = Document::new(context, id);
        let canonicalized = json_canon::to_string(&document).unwrap();
        assert_eq!(
            canonicalized,
            r#"{"@context":"https://www.w3.org/ns/did/v1","assertionMethod":[],"authentication":[],"id":"did:example:123456789abcdefghi","keyAgreement":[],"service":[],"verificationMethod":[]}"#
        );
    }

    #[test]
    fn test_document_new_full() {
        let context = Context::SingleString("https://www.w3.org/ns/did/v1".to_owned());
        let id = "did:example:123456789abcdefghi".to_string();

        // Generate key for verification method
        let my_string = String::from("Sample seed bytes of thirtytwo!b");
        let seed: &[u8] = my_string.as_bytes();
        let eddsa_key_pair = Ed25519KeyPair::new_with_seed(seed).unwrap();
        let ecdh_key_pair = eddsa_key_pair.get_x25519().unwrap();

        let mut private_eddsa_vm = VerificationMethod::new(
            "did:example:123456789abcdefghi#keys-1".to_string(),
            "Ed25519VerificationKey2018".to_string(),
            "did:example:123456789abcdefghi".to_string(),
        );
        let mut public_eddsa_vm = private_eddsa_vm.clone();

        let eddsa_private_key_multibase = multibase::encode(Base58Btc, eddsa_key_pair.private_key_bytes().unwrap());
        private_eddsa_vm.private_key = Some(Multibase(eddsa_private_key_multibase));

        let eddsa_public_key_multibase = multibase::encode(Base58Btc, eddsa_key_pair.public_key_bytes().unwrap());
        public_eddsa_vm.public_key = Some(Multibase(eddsa_public_key_multibase));

        let mut private_ecdh_vm = VerificationMethod::new(
            "did:example:123456789abcdefghi#keys-2".to_string(),
            "X25519KeyAgreementKey2019".to_string(),
            "did:example:123456789abcdefghi".to_string(),
        );
        let mut public_ecdh_vm = private_ecdh_vm.clone();

        let ecdh_private_key_multibase = multibase::encode(Base58Btc, ecdh_key_pair.private_key_bytes().unwrap());
        private_ecdh_vm.private_key = Some(Multibase(ecdh_private_key_multibase));

        let ecdh_public_key_multibase = multibase::encode(Base58Btc, ecdh_key_pair.public_key_bytes().unwrap());
        public_ecdh_vm.public_key = Some(Multibase(ecdh_public_key_multibase));

        let private_verification_method = Some(vec![private_eddsa_vm, private_ecdh_vm]);
        let public_verification_method = Some(vec![public_eddsa_vm, public_ecdh_vm]);

        let authentication = Some(vec![VerificationMethodType::Reference(
            "did:example:123456789abcdefghi#keys-1".to_string(),
        )]);
        let assertion_method = Some(vec![VerificationMethodType::Reference(
            "did:example:123456789abcdefghi#keys-1".to_string(),
        )]);
        let key_agreement = Some(vec![VerificationMethodType::Reference(
            "did:example:123456789abcdefghi#keys-2".to_string(),
        )]);

        let srv = Service::new(
            "did:example:123456789abcdefghi#keys-1".to_string(),
            "did-communication".to_string(),
            Value::String("https://example.com".to_string()),
        );
        let service = Some(vec![srv]);

        let private_document = Document::new_full(
            context.clone(),
            id.clone(),
            authentication.clone(),
            assertion_method.clone(),
            key_agreement.clone(),
            private_verification_method,
            service.clone(),
        );
        let public_document = Document::new_full(
            context,
            id,
            authentication,
            assertion_method,
            key_agreement,
            public_verification_method,
            service,
        );

        let canonicalized_private_document = json_canon::to_string(&private_document).unwrap();
        let canonicalized_public_document = json_canon::to_string(&public_document).unwrap();

        let expected_private_document = std::fs::read_to_string("test_resources/didkit_test_document_new_full_private.json").unwrap();
        let expected_public_document = std::fs::read_to_string("test_resources/didkit_test_document_new_full_public.json").unwrap();

        assert_eq!(expected_private_document, canonicalized_private_document);
        assert_eq!(expected_public_document, canonicalized_public_document);
    }
}
