use message_api::MessagePlugin;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub(crate) static DIDCOMM_PLUGINS: Lazy<Vec<Arc<dyn MessagePlugin>>> = Lazy::new(|| {
    vec![
        #[cfg(feature = "routing")]
        Arc::new(forward::plugin::RoutingProtocol),
        #[cfg(feature = "pickup")]
        Arc::new(pickup::plugin::PickupProtocol),
        #[cfg(feature = "trust-ping")]
        Arc::new(trust_ping::plugin::TrustPingProtocol),
        #[cfg(feature = "mediator-coordination")]
        Arc::new(mediator_coordination::plugin::MediatorCoordinationProtocol),
    ]
});
