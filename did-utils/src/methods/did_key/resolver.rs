use async_trait::async_trait;

use crate::{
    ldmodel::Context,
    methods::{
        errors::DIDResolutionError,
        traits::DIDResolver,
    },
};

use crate::methods::resolution::{
    DIDResolutionMetadata,
    DIDResolutionOptions,
    MediaType,
    ResolutionOutput,
};

use super::DIDKeyMethod;

#[async_trait]
impl DIDResolver for DIDKeyMethod {
    
    /// Resolves a DID using the did:key method.
    /// 
    /// # Arguments
    /// 
    /// * `did` - The DID address to resolve.
    /// 
    /// # Returns
    /// 
    /// A `ResolutionOutput` struct containing the resolved DID document and metadata.
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput {
        let context = Context::SingleString(String::from("https://w3id.org/did-resolution/v1"));

        match self.expand(did) {
            Ok(diddoc) => ResolutionOutput {
                context,
                did_document: Some(diddoc),
                did_resolution_metadata: Some(DIDResolutionMetadata {
                    error: None,
                    content_type: Some(MediaType::DidLdJson.to_string()),
                    additional_properties: None,
                }),
                did_document_metadata: None,
                additional_properties: None,
            },
            Err(err) => ResolutionOutput {
                context,
                did_document: None,
                did_resolution_metadata: Some(DIDResolutionMetadata {
                    error: Some(if !did.starts_with("did:key:") {
                        DIDResolutionError::MethodNotSupported
                    } else {
                        err
                    }),
                    content_type: None,
                    additional_properties: None,
                }),
                did_document_metadata: None,
                additional_properties: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::resolution::DereferencingOptions;
    use serde_json::Value;

    #[async_std::test]
    async fn test_did_key_resolution_with_encryption_derivation() {
        let did_method = DIDKeyMethod {
            enable_encryption_key_derivation: true,
            ..Default::default()
        };

        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let expected: Value = serde_json::from_str(
            r#"{
                "@context": "https://w3id.org/did-resolution/v1",
                "didDocument": {
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
                },
                "didResolutionMetadata": {
                    "contentType": "application/did+ld+json"
                },
                "didDocumentMetadata": null
            }"#,
        )
        .unwrap();

        let output = did_method.resolve(did, &DIDResolutionOptions::default()).await;

        assert_eq!(
            json_canon::to_string(&output).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[async_std::test]
    async fn test_did_key_resolution_fails_as_expected() {
        let did_method = DIDKeyMethod::default();

        let did = "did:key:Z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let expected: Value = serde_json::from_str(
            r#"{
                "@context": "https://w3id.org/did-resolution/v1",
                "didDocument": null,
                "didResolutionMetadata": {
                    "error": "invalidDid"
                },
                "didDocumentMetadata": null
            }"#,
        )
        .unwrap();

        let output = did_method.resolve(did, &DIDResolutionOptions::default()).await;

        assert_eq!(
            json_canon::to_string(&output).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[async_std::test]
    async fn test_dereferencing_did_key_url() {
        let did_method = DIDKeyMethod {
            enable_encryption_key_derivation: true,
            ..Default::default()
        };

        let did_url = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p";
        let expected: Value = serde_json::from_str(
            r#"{
                "@context": "https://w3id.org/did-resolution/v1",
                "content": {
                    "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p",
                    "type": "X25519KeyAgreementKey2020",
                    "controller": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                    "publicKeyMultibase": "z6LSj72tK8brWgZja8NLRwPigth2T9QRiG1uH9oKZuKjdh9p"
                },
                "dereferencingMetadata": {
                    "contentType": "application/json"
                },
                "contentMetadata": null
            }"#,
        )
        .unwrap();

        let output = did_method.dereference(did_url, &DereferencingOptions::default()).await;

        assert_eq!(
            json_canon::to_string(&output).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }
}
