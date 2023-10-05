pub mod container;
pub mod traits;

use lazy_static::lazy_static;
use traits::Plugin;

#[cfg(feature = "plugin-index")]
mod index;

#[cfg(feature = "plugin-did_endpoint")]
mod did_endpoint;

lazy_static! {
    pub static ref PLUGINS: Vec<Box<dyn Plugin>> = vec![
        #[cfg(feature = "plugin-index")]
        Box::<index::IndexPlugin>::default(),
        #[cfg(feature = "plugin-did_endpoint")]
        Box::<did_endpoint::DidEndpointPlugin>::default(),
    ];
}
