use async_trait::async_trait;
use axum::response::Response;
use didcomm::Message;
use shared::state::AppState;
use std::{collections::HashMap, sync::Arc};

/// Defines a handler that processes `DIDComm` messages.
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Process a `DIDComm` message.
    ///
    /// Returns an optional message to be sent back to the sender or an error response.
    async fn handle(&self, state: Arc<AppState>, msg: Message)
        -> Result<Option<Message>, Response>;
}

/// A router that maps `DIDComm` message types to their corresponding handlers.
#[derive(Default, Clone)]
pub struct MessageRouter {
    handlers: HashMap<String, Arc<dyn MessageHandler>>,
}

impl MessageRouter {
    /// Creates a new [`MessageRouter`].
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Registers a handler for a specific message type.
    pub fn register<F>(mut self, msg: &str, f: F) -> Self
    where
        F: MessageHandler + 'static,
    {
        self.handlers.insert(msg.to_string(), Arc::new(f));
        self
    }

    /// Merges another [`MessageRouter`] into this one.
    pub fn merge(mut self, other: Self) -> Self {
        self.handlers.extend(other.handlers);
        self
    }

    /// Returns the handler for a specific message type if it is registered.
    pub fn get_handler(&self, msg: &str) -> Option<&Arc<dyn MessageHandler>> {
        self.handlers.get(msg)
    }

    /// Returns a list of all registered message types.
    pub fn messages_types(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

pub trait MessagePlugin: Send + Sync {
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Return a mapping of message types to handlers
    fn didcomm_routes(&self) -> MessageRouter;
}
