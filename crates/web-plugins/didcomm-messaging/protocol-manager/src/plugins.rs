use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use plugin_api::Plugin;

lazy_static! {
    pub(crate) static ref PLUGINS: Vec<Arc<Mutex<dyn Plugin + 'static>>> = vec![
        #[cfg(feature = "protocol-forward")]
        // Arc::new(Mutex::new(index::IndexPlugin {})),
        #[cfg(feature = "protocol-pickup")]
        // Arc::new(Mutex::new(did_endpoint::plugin::DidEndpoint {})),
        #[cfg(feature = "protocole-mediator-coordination")]
        Arc::new(Mutex::new(oob_messages::plugin::OOBMessages {})),
    ];
}
