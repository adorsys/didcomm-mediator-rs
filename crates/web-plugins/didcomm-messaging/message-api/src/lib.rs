use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

type MessageHandler<S, M, R> = fn(Arc<S>, M) -> Result<R, PluginError>;

#[derive(Debug, PartialEq)]
pub enum PluginError {
    InitError,
}

#[derive(Debug, Clone)]
pub struct MessageRouter<S, M, R>
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    R: Send + 'static,
{
    routes: HashMap<String, MessageHandler<S, M, R>>,
    _marker: PhantomData<(S, M, R)>,
}

impl<S, M, R> MessageRouter<S, M, R>
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    R: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn route(mut self, msg_type: &str, f: MessageHandler<S, M, R>) -> Self {
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
impl<S, M, R> Default for MessageRouter<S, M, R>
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    R: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait MessagePlugin<S, M, R>: Sync + Send
where
    S: Clone + Sync + Send + 'static,
    M: Send + 'static,
    R: Send + 'static,
{
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Provide initialization actions as needed
    fn mount(&mut self) -> Result<(), PluginError>;

    /// Revert initialization actions as needed
    fn unmount(&self) -> Result<(), PluginError>;

    /// Return a mapping of message types to handlers
    fn routes(&self) -> MessageRouter<S, M, R>;
}
