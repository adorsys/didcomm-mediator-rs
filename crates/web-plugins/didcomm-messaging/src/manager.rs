use std::{collections::HashSet, sync::Arc};

use crate::protocol::PROTOCOLS;
use message_api::{MessagePlugin, MessageRegistry};

#[derive(Debug, PartialEq)]
pub enum MessageContainerError {
    DuplicateEntry,
    Unloaded,
}

pub struct MessagePluginContainer<'a> {
    loaded: bool,
    collected_handlers: Vec<MessageRegistry>,
    protocol_plugins: &'a Vec<Arc<dyn MessagePlugin>>,
}

impl<'a> MessagePluginContainer<'a> {
    pub fn new() -> Self {
        Self {
            loaded: false,
            collected_handlers: vec![],
            protocol_plugins: &PROTOCOLS,
        }
    }

    pub fn find_handler(&self, name: &str) -> Option<Arc<dyn MessagePlugin>> {
        self.protocol_plugins
            .iter()
            .find_map(|handler| (handler.name() == name).then_some(handler.clone()))
    }

    pub fn load(&mut self) -> Result<(), MessageContainerError> {
        tracing::debug!("Loading protocol container");

        let mut seen_names = HashSet::new();
        for plugin in self.protocol_plugins.iter() {
            let plugin = plugin.clone();
            if !seen_names.insert(plugin.name().to_string()) {
                tracing::error!(
                    "found duplicate entry in DIDComm message registry: {}",
                    plugin.name()
                );
                return Err(MessageContainerError::DuplicateEntry);
            }
        }

        // Reset route collection
        self.collected_handlers.clear();

        // Collect handlers
        for plugin in self.protocol_plugins.iter() {
            tracing::info!("registering didcomm protocol: {}", plugin.name());
            self.collected_handlers.push(plugin.clone().handlers());
        }

        // Update loaded status
        self.loaded = true;
        tracing::debug!("protocol container loaded successfully");
        Ok(())
    }

    /// unload container protocols
    pub fn unload(&mut self) -> Result<(), MessageContainerError> {
        self.loaded = false;
        self.collected_handlers.clear();

        Ok(())
    }

    pub fn handlers(&self) -> Result<MessageRegistry, MessageContainerError> {
        if !self.loaded {
            return Err(MessageContainerError::Unloaded);
        }

        Ok(self
            .collected_handlers
            .iter()
            .fold(MessageRegistry::new(), |acc: MessageRegistry, e| {
                acc.merge(e.clone())
            }))
    }
}
