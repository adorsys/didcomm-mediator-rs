pub mod container;

use std::sync::{Arc,Mutex};

use lazy_static::lazy_static;
use server_plugin::Plugin;

#[cfg(feature = "plugin-index")]
mod index;

lazy_static! {
    pub static ref PLUGINS: Arc<Vec<Arc<Mutex<dyn Plugin + 'static + Send>>>> = Arc::new(vec![
        #[cfg(feature = "plugin-index")]
        Arc::new(Mutex::new(index::IndexPlugin::default())),
        #[cfg(feature = "plugin-did_endpoint")]
        Arc::new(Mutex::new(did_endpoint::plugin::DidEndpointPlugin::default())),
        #[cfg(feature = "plugin-oob_messages")]
        Arc::new(Mutex::new(oob_messages::plugin::OOBMessagesPlugin::default())),
    ]);
}
