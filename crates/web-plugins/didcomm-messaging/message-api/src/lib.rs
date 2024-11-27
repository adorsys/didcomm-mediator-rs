use async_trait::async_trait;
use axum::response::Response;
use didcomm::Message;
use shared::state::AppState;
use std::{collections::HashMap, sync::Arc};

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, state: Arc<AppState>, msg: Message)
        -> Result<Option<Message>, Response>;
}

#[derive(Default, Clone)]
pub struct MessageRouter {
    handlers: HashMap<String, Arc<dyn MessageHandler>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<F>(mut self, msg: &str, f: F) -> Self
    where
        F: MessageHandler + 'static,
    {
        self.handlers.insert(msg.to_string(), Arc::new(f));
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.handlers.extend(other.handlers);
        self
    }

    pub fn get_handler(&self, msg: &str) -> Option<&Arc<dyn MessageHandler>> {
        self.handlers.get(msg)
    }

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
