#![allow(dead_code)]

use crate::protocols::DIDCOMM_PLUGINS;
use message_api::{MessagePlugin, MessageRouter};
use std::{collections::HashSet, sync::Arc};

#[derive(Debug, PartialEq)]
pub(crate) enum MessageContainerError {
    DuplicateEntry,
    Unloaded,
}

pub(crate) struct MessagePluginContainer<'a> {
    loaded: bool,
    collected_routes: Vec<MessageRouter>,
    message_plugins: &'a Vec<Arc<dyn MessagePlugin>>,
}

impl<'a> MessagePluginContainer<'a> {
    pub(crate) fn new() -> Self {
        Self {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &DIDCOMM_PLUGINS,
        }
    }

    pub(crate) fn find_handler(&self, name: &str) -> Option<Arc<dyn MessagePlugin>> {
        self.message_plugins
            .iter()
            .find_map(|handler| (handler.name() == name).then_some(handler.clone()))
    }

    pub(crate) fn load(&mut self) -> Result<(), MessageContainerError> {
        tracing::debug!("Loading DIDCcomm protocols container");

        let mut seen_names = HashSet::new();
        for protocol in self.message_plugins.iter() {
            if !seen_names.insert(protocol.name()) {
                tracing::error!(
                    "found duplicate entry in DIDComm protocols container: {}",
                    protocol.name()
                );
                return Err(MessageContainerError::DuplicateEntry);
            }
        }

        // Reset route collection
        self.collected_routes.clear();

        // Collect didcomm messages routes
        for protocol in self.message_plugins.iter() {
            tracing::info!("registering didcomm protocol: {}", protocol.name());
            self.collected_routes.push(protocol.didcomm_routes());
        }

        // Update loaded status
        self.loaded = true;
        tracing::debug!("DIDComm protocols container loaded successfully");
        Ok(())
    }

    /// unload container protocols
    pub(crate) fn unload(&mut self) -> Result<(), MessageContainerError> {
        self.loaded = false;
        self.collected_routes.clear();

        Ok(())
    }

    pub(crate) fn didcomm_routes(&self) -> Result<MessageRouter, MessageContainerError> {
        if !self.loaded {
            return Err(MessageContainerError::Unloaded);
        }

        Ok(self
            .collected_routes
            .iter()
            .fold(MessageRouter::new(), |acc: MessageRouter, e| {
                acc.merge(e.clone())
            }))
    }
}
