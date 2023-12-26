use async_trait::async_trait;

use crate::{
    ldmodel::Context,
    methods::{
        errors::DIDResolutionError,
        traits::{DIDResolutionMetadata, DIDResolutionOptions, DIDResolver, MediaType, ResolutionOutput},
    },
};

use super::DIDPeerMethod;

#[async_trait]
impl DIDResolver for DIDPeerMethod {
    /// Resolves a DID address into its corresponding DID document.
    async fn resolve(&self, did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput {
        todo!()
    }
}
