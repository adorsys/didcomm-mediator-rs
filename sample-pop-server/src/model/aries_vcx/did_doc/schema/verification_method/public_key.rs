use crate::model::aries_vcx::did_doc;

use std::str::FromStr;

use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};

use did_doc::{
    error::DidDocumentBuilderError,
    schema::types::{jsonwebkey::JsonWebKey, multibase::Multibase},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum PublicKeyField {
    #[serde(rename_all = "camelCase")]
    Multibase { public_key_multibase: String },
    #[serde(rename_all = "camelCase")]
    Jwk { public_key_jwk: JsonWebKey },
    #[serde(rename_all = "camelCase")]
    Base58 { public_key_base58: String },
    #[serde(rename_all = "camelCase")]
    Base64 { public_key_base64: String },
    #[serde(rename_all = "camelCase")]
    Hex { public_key_hex: String },
    #[serde(rename_all = "camelCase")]
    Pem { public_key_pem: String },
    #[serde(rename_all = "camelCase")]
    Pgp { public_key_pgp: String },
}

impl PublicKeyField {
    pub fn key_decoded(&self) -> Result<Vec<u8>, DidDocumentBuilderError> {
        match self {
            PublicKeyField::Multibase {
                public_key_multibase,
            } => {
                let multibase = Multibase::from_str(public_key_multibase)?;
                Ok(multibase.as_ref().to_vec())
            }
            PublicKeyField::Jwk { public_key_jwk } => public_key_jwk.to_vec(),
            PublicKeyField::Base58 { public_key_base58 } => {
                Ok(bs58::decode(public_key_base58).into_vec()?)
            }
            PublicKeyField::Base64 { public_key_base64 } => {
                Ok(general_purpose::STANDARD_NO_PAD.decode(public_key_base64.as_bytes())?)
            }
            PublicKeyField::Hex { public_key_hex } => Ok(hex::decode(public_key_hex)?),
            PublicKeyField::Pem { public_key_pem } => {
                Ok(pem::parse(public_key_pem.as_bytes())?.contents().to_vec())
            }
            PublicKeyField::Pgp { public_key_pgp: _ } => Err(
                DidDocumentBuilderError::UnsupportedPublicKeyField("publicKeyPgp"),
            ),
        }
    }
}
