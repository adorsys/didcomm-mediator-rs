use message_api::{MessagePlugin, MessageRouter, PluginError};

#[derive(Default)]
pub struct PickupProtocols;

impl<S, M, R> MessagePlugin<S, M, R> for PickupProtocols
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    R: Send + 'static,
{
    fn name(&self) -> &'static str {
        "pickup-protocol"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> MessageRouter<S, M, R> {
        let mut router = MessageRouter::new();
        // router.route("/PickupProtocol", self.collected_routes);
        router
    }
}
