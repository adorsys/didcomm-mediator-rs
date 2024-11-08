use std::sync::{Arc, Mutex};

use axum::response::Response;
use didcomm::Message;
use message_api::MessagePlugin;
use once_cell::sync::Lazy;
use shared::state::AppState;


pub static PROTOCOLS: Lazy<Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>>> = Lazy::new(|| {
    vec![
        #[cfg(feature = "forward-protocol")]
        Arc::new(Mutex::new(vec![Box::new(forward::plugin::ForwardProtocols::default())])),
        #[cfg(feature = "pickup-protocol")]
        Arc::new(Mutex::new(vec![Box::new(pickup::plugin::PickupProtocols::default())])),
        #[cfg(feature = "mediator-coordination-protocol")]
        Arc::new(Mutex::new(vec![Box::new(mediator_coordination::plugin::MediatorCoordination::default())])),
    ]
});
