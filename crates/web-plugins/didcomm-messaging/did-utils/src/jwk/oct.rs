use serde::{Deserialize, Serialize};

use super::secret::Secret;

/// A symmetric octet key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Oct {
    /// The symmetric key.
    pub k: Secret,
}
