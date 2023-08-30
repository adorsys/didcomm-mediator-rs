use async_trait::async_trait;
use ssi::{
    did::Document,
    did_resolve::{
        DIDResolver, DocumentMetadata, ResolutionInputMetadata, ResolutionMetadata,
        ERROR_NOT_FOUND, TYPE_DID_LD_JSON,
    },
};

pub struct StaticResolver<'a> {
    diddoc: &'a Document,
}

impl<'a> StaticResolver<'a> {
    pub fn new(diddoc: &'a Document) -> Self {
        Self { diddoc }
    }
}

#[async_trait]
impl<'a> DIDResolver for StaticResolver<'a> {
    async fn resolve(
        &self,
        did: &str,
        _input_metadata: &ResolutionInputMetadata,
    ) -> (
        ResolutionMetadata,
        Option<Document>,
        Option<DocumentMetadata>,
    ) {
        if did == self.diddoc.id {
            (
                ResolutionMetadata {
                    content_type: Some(TYPE_DID_LD_JSON.to_string()),
                    ..Default::default()
                },
                Some(self.diddoc.clone()),
                Some(DocumentMetadata::default()),
            )
        } else {
            (
                ResolutionMetadata {
                    error: Some(ERROR_NOT_FOUND.to_string()),
                    content_type: None,
                    property_set: None,
                },
                None,
                None,
            )
        }
    }

    async fn resolve_representation(
        &self,
        did: &str,
        _input_metadata: &ResolutionInputMetadata,
    ) -> (ResolutionMetadata, Vec<u8>, Option<DocumentMetadata>) {
        if did == self.diddoc.id {
            (
                ResolutionMetadata {
                    error: None,
                    content_type: Some(TYPE_DID_LD_JSON.to_string()),
                    property_set: None,
                },
                serde_json::to_string(&self.diddoc)
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
                Some(DocumentMetadata::default()),
            )
        } else {
            (
                ResolutionMetadata {
                    error: Some(ERROR_NOT_FOUND.to_string()),
                    content_type: None,
                    property_set: None,
                },
                Vec::new(),
                None,
            )
        }
    }
}
