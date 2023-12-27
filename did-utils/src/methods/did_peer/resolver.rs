use async_trait::async_trait;

use crate::methods::traits::{DIDResolutionOptions, DIDResolver, ResolutionOutput};

use super::DIDPeerMethod;

#[async_trait]
impl DIDResolver for DIDPeerMethod {
    /// Resolves a DID address into its corresponding DID document.
    async fn resolve(&self, _did: &str, _options: &DIDResolutionOptions) -> ResolutionOutput {
        todo!()
    }
}
