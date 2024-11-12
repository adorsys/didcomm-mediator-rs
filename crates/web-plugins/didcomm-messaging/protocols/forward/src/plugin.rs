use message_api::{MessagePlugin, MessageRouter, PluginError};

#[derive(Default)]
pub struct ForwardProtocols;

impl<S, M, R> MessagePlugin<S, M, R> for ForwardProtocols
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    R: Send + 'static,
{
    fn name(&self) -> &'static str {
        "forward"
    }
    fn mount(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> MessageRouter<S, M, R> {
        let mut router = MessageRouter::new();
        // router.route("/forwardProtocol", self.collected_routes);
        router
    }
}
