use crate::model::aries_vcx::did_doc;

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use multibase::{decode, Base};
use serde::{Deserialize, Serialize};

use did_doc::error::DidDocumentBuilderError;

// https://datatracker.ietf.org/doc/html/draft-multiformats-multibase-07
#[derive(Clone, Debug, PartialEq)]
pub struct Multibase {
    base: Base,
    bytes: Vec<u8>,
}

impl Multibase {
    pub fn new(multibase: String) -> Result<Self, DidDocumentBuilderError> {
        let (base, bytes) = decode(multibase).map_err(|err| {
            DidDocumentBuilderError::InvalidInput(format!("Invalid multibase key: {}", err))
        })?;
        Ok(Self { base, bytes })
    }

    pub fn base_to_multibase(base: Base, encoded: &str) -> Self {
        let multibase_encoded = format!("{}{}", base.code(), encoded);
        Self {
            base,
            bytes: multibase_encoded.as_bytes().to_vec(),
        }
    }
}

impl Serialize for Multibase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.base.encode(&self.bytes))
    }
}

impl<'de> Deserialize<'de> for Multibase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Multibase {
    type Err = DidDocumentBuilderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl Display for Multibase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base.encode(&self.bytes))
    }
}

impl AsRef<[u8]> for Multibase {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
