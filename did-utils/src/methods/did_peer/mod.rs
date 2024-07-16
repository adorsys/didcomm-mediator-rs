//! The [`did:peer`] DID method is designed to be used independent of any central source of truth,
//! and is intended to be cheap, fast, scalable, and secure. It is suitable for most private
//! relationships between people, organizations, and things.
//!
//! [`did:peer`]: https://identity.foundation/peer-did-method-spec/

mod errors;
mod resolver;
mod util;
pub(super) mod method;
