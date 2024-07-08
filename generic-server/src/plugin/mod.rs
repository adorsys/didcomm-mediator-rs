pub mod container;

use std::sync::Mutex;

use lazy_static::lazy_static;
use server_plugin::Plugin;

#[cfg(feature = "plugin-index")]
mod index;

lazy_static! {
    pub static ref PLUGINS: Mutex<Vec<Box<dyn Plugin + 'static + Send>>> = Mutex::new(vec![
        #[cfg(feature = "plugin-index")]
        Box::<index::IndexPlugin>::default(),
        #[cfg(feature = "plugin-did_endpoint")]
        Box::<did_endpoint::plugin::DidEndpointPlugin>::default(),
        #[cfg(feature = "plugin-oob_messages")]
        Box::<oob_messages::plugin::OOBMessagesPlugin>::default(),
    ]);
}
