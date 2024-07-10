pub mod container;

use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use server_plugin::Plugin;

#[cfg(feature = "plugin-index")]
mod index;

lazy_static! {
    pub static ref PLUGINS: Arc<Vec<Arc<Mutex<dyn Plugin + 'static>>>> = Arc::new(vec![
        #[cfg(feature = "plugin-index")]
        Arc::<Mutex::<index::IndexPlugin>>::default(),
        #[cfg(feature = "plugin-did_endpoint")]
        Arc::<Mutex::<did_endpoint::plugin::DidEndpointPlugin>>::default(),
        #[cfg(feature = "plugin-oob_messages")]
        Arc::<Mutex::<oob_messages::plugin::OOBMessagesPlugin>>::default(),
    ]);
}
