use crate::model::aries_vcx::did_doc;

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use did_doc::error::DidDocumentBuilderError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Uri(uniresid::Uri);

impl Uri {
    pub fn new(uri: &str) -> Result<Self, DidDocumentBuilderError> {
        Ok(Self(uniresid::Uri::try_from(uri).map_err(|e| {
            DidDocumentBuilderError::InvalidInput(format!("Invalid URI: {}", e))
        })?))
    }
}

impl FromStr for Uri {
    type Err = DidDocumentBuilderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<uniresid::Uri> for Uri {
    fn as_ref(&self) -> &uniresid::Uri {
        &self.0
    }
}
