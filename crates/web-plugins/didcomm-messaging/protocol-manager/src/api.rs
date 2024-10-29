use std::{
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::Arc,
};

use axum::response::Response;
use didcomm::Message;

#[derive(Debug, PartialEq)]
pub enum PluginError {
    InitError,
}

pub type Handler = fn(state: Arc<AppState>, msg: Message) -> Result<Message, Response>;

#[derive(Debug)]
pub struct ProtocolRouter {
    pub routes: HashMap<String, Handler>,
}

impl ProtocolRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn routes(self, msg_type: &str, handler: Handler) -> Self {
        Self { routes: self.routes.insert(msg_type.to_string(), handler), }
    }
}

pub trait Plugin: Sync + Send {
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Provide initialization actions as needed
    fn mount(&mut self) -> Result<(), PluginError>;

    /// Revert initialization actions as needed
    fn unmount(&self) -> Result<(), PluginError>;

    /// Return a mapping of message types to handlers
    fn routes(&self) -> ProtocolRouter;
}

impl Eq for dyn Plugin {}

impl PartialEq for dyn Plugin {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Hash for dyn Plugin {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name().hash(state)
    }
}
