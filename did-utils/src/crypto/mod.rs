pub mod traits;
pub mod utils;
pub mod ed25519;

pub struct AsymmetricKey<P, S> {
    pub public_key: P,
    pub secret_key: Option<S>,
}
