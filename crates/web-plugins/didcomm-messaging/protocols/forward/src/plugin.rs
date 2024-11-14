use std::sync::Arc;

use didcomm::Message;
use message_api::{Handler, MessagePlugin, MessageRouter, PluginError};
use shared::state::AppState;

use crate::ForwardError;

#[derive(Debug)]
pub struct ForwardPlugin;

impl Handler for ForwardPlugin {
    type State = Arc<AppState>;
    type Message = Option<Message>;
    type Error = ForwardError;

    async fn handle(&self, state: Self::State, msg: Message) -> Result<Self::Message, Self::Error> {
        crate::web::handler::mediator_forward_process(state, msg).await
    }
}

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
