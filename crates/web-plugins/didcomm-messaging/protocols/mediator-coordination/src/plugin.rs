use message_api::{MessagePlugin, MessageRouter, PluginError};

#[derive(Default)]
pub struct MediatorCoordination;

impl<S, M, R> MessagePlugin<S, M, R> for MediatorCoordination
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    R: Send + 'static,
{
    fn name(&self) -> &'static str {
        "mediator-coordination"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> MessageRouter<S, M, R> {
        MessageRouter::new()
    }
}
