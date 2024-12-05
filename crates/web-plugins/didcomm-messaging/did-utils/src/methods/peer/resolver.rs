use async_trait::async_trait;

use super::method::DidPeer;
use crate::{
    ldmodel::Context,
    methods::{
        errors::DIDResolutionError,
        resolution::{DIDResolutionMetadata, DIDResolutionOptions, MediaType, ResolutionOutput},
        traits::DIDResolver,
    },
};

#[async_trait]
impl DIDResolver for DidPeer {
    /// Resolves a `did:peer` address to a DID document.
    ///
    /// # Example
    ///
    /// ```
    /// use did_utils::methods::{DIDResolver, DidPeer, DIDResolutionOptions};
    ///
    /// # async fn example_resolve_did_peer() {
    /// // create new peer did resolver
    /// let did_peer_resolver = DidPeer::new();
    /// let did = "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc";
    /// // resolve the did
    /// let output = did_peer_resolver.resolve(did, &DIDResolutionOptions::default()).await;
    /// # }
    /// ```
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
                    error: Some(if !did.starts_with("did:peer:") {
                        DIDResolutionError::MethodNotSupported
                    } else {
                        err.into()
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
    use serde_json::Value;

    #[async_std::test]
    async fn test_did_peer_resolution() {
        let did_method = DidPeer::new();

        let did = "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let expected: Value = serde_json::from_str(
            r##"{
                "@context": "https://w3id.org/did-resolution/v1",
                "didDocument": {
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
                },
                "didResolutionMetadata": {
                    "contentType": "application/did+ld+json"
                },
                "didDocumentMetadata": null
            }"##,
        )
        .unwrap();

        let output = did_method.resolve(did, &DIDResolutionOptions::default()).await;
        assert_eq!(
            json_canon::to_string(&output).unwrap(),   //
            json_canon::to_string(&expected).unwrap(), //
        );
    }

    #[async_std::test]
    async fn test_did_peer_resolution_fails_on_invalid_did() {
        let did_method = DidPeer::new();
        let did = concat!(
            "did:peer:2",
            ".Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc",
            // {"s":"http://example.com/xyz","t":"dm" (missing closing brace)
            ".SeyJzIjoiaHR0cDovL2V4YW1wbGUuY29tL3h5eiIsInQiOiJkbSI",
        );

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
    async fn test_did_peer_resolution_fails_on_unsupported_peer_did_submethod() {
        let did_method = DidPeer::new();
        let did = "did:peer:1zQmbEB1EqP7PnNVaHiSpXhkatAA6kNyQK9mWkvrMx2eckgq";

        let expected: Value = serde_json::from_str(
            r#"{
                "@context": "https://w3id.org/did-resolution/v1",
                "didDocument": null,
                "didResolutionMetadata": {
                    "error": "methodNotSupported"
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
}
