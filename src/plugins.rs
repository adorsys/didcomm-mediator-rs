pub(crate) mod handler;
#[cfg(feature = "plugin-index")]
pub(crate) mod index;

pub use handler::{PluginContainer, PluginContainerError};

use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use plugin_api::Plugin;

lazy_static! {
    pub(crate) static ref PLUGINS: Vec<Arc<Mutex<dyn Plugin + 'static>>> = vec![
        #[cfg(feature = "plugin-index")]
        Arc::new(Mutex::new(index::IndexPlugin {})),
        #[cfg(feature = "plugin-did_endpoint")]
        Arc::new(Mutex::new(did_endpoint::plugin::DidEndpoint {})),
        #[cfg(feature = "plugin-oob_messages")]
        Arc::new(Mutex::new(oob_messages::plugin::OOBMessages {})),
        #[cfg(feature = "plugin-plugins")]
        Arc::new(Mutex::new(plugins::plugin::MediatorCoordination::default())),
    ];
}
