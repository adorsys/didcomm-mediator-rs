use crate::model::aries_vcx::did_parser;
use crate::model::aries_vcx::did_doc;

use did_parser::{Did, DidUrl};
use serde::{Deserialize, Serialize};

use did_doc::schema::types::jsonwebkey::JsonWebKey;

use super::{public_key::PublicKeyField, VerificationMethodType};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    id: DidUrl,
    controller: Did,
    #[serde(rename = "type")]
    verification_method_type: VerificationMethodType,
    #[serde(flatten)]
    public_key: PublicKeyField,
}

impl VerificationMethod {
    pub fn builder(
        id: DidUrl,
        controller: Did,
        verification_method_type: VerificationMethodType,
    ) -> IncompleteVerificationMethodBuilder {
        IncompleteVerificationMethodBuilder::new(id, controller, verification_method_type)
    }

    pub fn id(&self) -> &DidUrl {
        &self.id
    }

    pub fn controller(&self) -> &Did {
        &self.controller
    }

    pub fn verification_method_type(&self) -> &VerificationMethodType {
        &self.verification_method_type
    }

    pub fn public_key(&self) -> &PublicKeyField {
        &self.public_key
    }
}

#[derive(Debug, Clone)]
pub struct IncompleteVerificationMethodBuilder {
    id: DidUrl,
    controller: Did,
    verification_method_type: VerificationMethodType,
}

#[derive(Debug, Clone)]
pub struct CompleteVerificationMethodBuilder {
    id: DidUrl,
    controller: Did,
    verification_method_type: VerificationMethodType,
    public_key: Option<PublicKeyField>,
}

impl IncompleteVerificationMethodBuilder {
    pub fn new(
        id: DidUrl,
        controller: Did,
        verification_method_type: VerificationMethodType,
    ) -> Self {
        Self {
            id,
            verification_method_type,
            controller,
        }
    }

    pub fn add_public_key_multibase(
        self,
        public_key_multibase: String,
    ) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key: Some(PublicKeyField::Multibase {
                public_key_multibase,
            }),
        }
    }

    pub fn add_public_key_jwk(
        self,
        public_key_jwk: JsonWebKey,
    ) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key: Some(PublicKeyField::Jwk { public_key_jwk }),
        }
    }

    pub fn add_public_key_base58(
        self,
        public_key_base58: String,
    ) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key: Some(PublicKeyField::Base58 { public_key_base58 }),
        }
    }

    pub fn add_public_key_base64(
        self,
        public_key_base64: String,
    ) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key: Some(PublicKeyField::Base64 { public_key_base64 }),
        }
    }

    pub fn add_public_key_hex(self, public_key_hex: String) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key: Some(PublicKeyField::Hex { public_key_hex }),
        }
    }

    pub fn add_public_key_pem(self, public_key_pem: String) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key: Some(PublicKeyField::Pem { public_key_pem }),
        }
    }
}

impl CompleteVerificationMethodBuilder {
    pub fn build(self) -> VerificationMethod {
        VerificationMethod {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key: self.public_key.unwrap(), // SAFETY: The builder will always set the public key
        }
    }
}
