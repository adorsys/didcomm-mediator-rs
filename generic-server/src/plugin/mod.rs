pub mod container;

use lazy_static::lazy_static;
use server_plugin::Plugin;

#[cfg(feature = "plugin-index")]
mod index;

lazy_static! {
    pub static ref PLUGINS: Vec<Box<dyn Plugin>> = vec![
        #[cfg(feature = "plugin-index")]
        Box::<index::IndexPlugin>::default(),
        #[cfg(feature = "plugin-did_endpoint")]
        Box::<did_endpoint::plugin::DidEndpointPlugin>::default(),
        #[cfg(feature = "plugin-mediator_coordination")]
        Box::<mediator_coordination::plugin::MediatorCoordinationPlugin>::default(),
    ];
}
