use crate::{ key::key::Key, key::prm::Parameters };
extern crate alloc;
use serde::{ Deserialize, Serialize };

/// A set of JSON Web Keys.
///
/// This type is defined in [RFC7517 Section 5].
///
/// [RFC7517 Section 5]: https://datatracker.ietf.org/doc/html/rfc7517#section-5
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwkSet {
    /// The keys in the set.
    pub keys: alloc::vec::Vec<Jwk>,
}

/// A JSON Web Key.
///
/// This type is defined in [RFC7517 Section 4].
///
/// [RFC7517 Section 4]: https://datatracker.ietf.org/doc/html/rfc7517#section-4
/// To - do: take it out of here:
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[derive(Eq)]
pub struct Jwk {
    #[serde(flatten)]
    pub key_type: Key,
    /// The key parameters.
    #[serde(flatten)]
    pub prm: Parameters,
}
