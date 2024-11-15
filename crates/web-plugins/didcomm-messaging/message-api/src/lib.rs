use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::response::Response;
use didcomm::Message;
use shared::state::AppState;

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, state: Arc<AppState>, msg: Message)
        -> Result<Option<Message>, Response>;
}

#[derive(Clone)]
pub struct MessageRegistry {
    handlers: HashMap<String, Arc<dyn MessageHandler>>,
}

impl MessageRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<F: MessageHandler + 'static>(mut self, msg: &str, f: F) -> Self {
        self.handlers.insert(msg.to_string(), Arc::new(f));
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.handlers.extend(other.handlers);
        self
    }

    pub fn get(&self, msg: &str) -> Option<&Arc<dyn MessageHandler>> {
        self.handlers.get(msg)
    }
}

pub trait MessagePlugin {
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Return a mapping of message types to handlers
    fn handlers(&self) -> MessageRegistry;
}
