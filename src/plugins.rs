#[cfg(feature = "plugin-index")]
pub(crate) mod index;
pub(crate) mod manager;

pub use manager::{PluginContainer, PluginContainerError};

use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use plugin_api::Plugin;

lazy_static! {
    pub(crate) static ref PLUGINS: Vec<Arc<Mutex<dyn Plugin + 'static>>> = vec![
        #[cfg(feature = "plugin-index")]
        Arc::new(Mutex::new(index::IndexPlugin {})),
        #[cfg(feature = "plugin-did_endpoint")]
        Arc::new(Mutex::new(did_endpoint::plugin::DidEndpoint::default())),
        #[cfg(feature = "plugin-oob_messages")]
        Arc::new(Mutex::new(oob_messages::plugin::OOBMessages::default())),
        #[cfg(feature = "plugin-didcomm_messaging")]
        Arc::new(Mutex::new(
            didcomm_messaging::plugin::DidcommMessaging::default()
        )),
    ];
}