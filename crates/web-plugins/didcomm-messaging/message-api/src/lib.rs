use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

use didcomm::Message;
use shared::state::AppState;

#[async_trait::async_trait]
pub trait Handler: Send + Sync + Debug + 'static {
    type State;
    type Message;
    type Error;

    async fn handle(&self, state: Self::State, msg: Self::Message) -> Result<Self::Message, Self::Error>;
}

#[derive(Debug, PartialEq)]
pub enum PluginError {
    InitError,
}

#[derive(Debug, Clone)]
pub struct MessageRouter<S = Arc<AppState>, M = Message, E = Box<anyhow::Error>>
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    E: Send + 'static,
{
    routes: HashMap<String, Arc<dyn Handler<State = S, Message = M, Error = E>>>,
    _marker: PhantomData<(S, M, E)>,
}

impl<S, M, E> MessageRouter<S, M, E>
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    E: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn route(mut self, msg_type: &str, f: impl Handler<State = S, Message = M, Error = E>) -> Self {
        self.routes.insert(msg_type.to_string(), Arc::new(f));
        self
    }

    pub fn merge(mut self, other: Self) {
        for (key, handler) in other.routes {
            self.routes.insert(key, handler);
        }
    }
}

// Implement Default for MessageRouter
impl<S, M, E> Default for MessageRouter<S, M, E>
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    E: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait MessagePlugin<S, M, E>: Sync + Send
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    E: Send + 'static,
{
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Provide initialization actions as needed
    fn mount(&mut self) -> Result<(), PluginError>;

    /// Revert initialization actions as needed
    fn unmount(&self) -> Result<(), PluginError>;

    /// Return a mapping of message types to handlers
    fn routes(&self) -> MessageRouter<S, M, E>;
}
