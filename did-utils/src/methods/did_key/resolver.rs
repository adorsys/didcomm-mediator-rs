use async_trait::async_trait;

use crate::methods::{
    errors::DIDResolutionError,
    traits::{DIDResolutionOptions, DIDResolver, ResolutionOutput}, DIDKeyMethod,
};

#[async_trait]
impl DIDResolver for DIDKeyMethod {
    /// Resolves a DID address into its corresponding DID document.
    async fn resolve(&self, _did: &str, _options: &DIDResolutionOptions) -> Result<ResolutionOutput, DIDResolutionError> {
        Err(DIDResolutionError::MethodNotSupported)
    }
}
