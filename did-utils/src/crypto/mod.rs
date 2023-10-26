pub mod ed25519;
pub mod sha256_hash;
pub mod traits;
pub mod utils;
pub mod x25519;

mod format;

pub struct AsymmetricKey<P, S> {
    pub public_key: P,
    pub secret_key: Option<S>,
}
