use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

type MessageHandler<S, M, E> = fn(S, M) -> Result<M, E>;

#[derive(Debug, PartialEq)]
pub enum PluginError {
    InitError,
}

#[derive(Debug, Clone)]
pub struct MessageRouter<S, M, E>
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    E: Send + 'static,
{
    routes: HashMap<String, MessageHandler<S, M, E>>,
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

    pub fn route(mut self, msg_type: &str, f: MessageHandler<S, M, E>) -> Self {
        self.routes.insert(msg_type.to_string(), f);
        self
    }

    pub fn merge(&mut self, other: &Self) {
        for (key, handler) in &other.routes {
            self.routes.insert(key.clone(), *handler);
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
