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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Real plugin implementations
    struct FirstPlugin;
    impl MessagePlugin for FirstPlugin {
        fn name(&self) -> &'static str {
            "first"
        }

        fn didcomm_routes(&self) -> MessageRouter {
            MessageRouter::new()
        }
    }

    struct SecondPlugin;
    impl MessagePlugin for SecondPlugin {
        fn name(&self) -> &'static str {
            "second"
        }

        fn didcomm_routes(&self) -> MessageRouter {
            MessageRouter::new()
        }
    }

    #[test]
    fn test_loading_plugins() {
        let plugins: Vec<Arc<dyn MessagePlugin>> =
            vec![Arc::new(FirstPlugin {}), Arc::new(SecondPlugin {})];

        let mut container = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &plugins,
        };

        assert!(container.load().is_ok());
        assert!(container.loaded);

        assert_eq!(container.collected_routes.len(), 2);

        assert!(container.find_handler("first").is_some());
        assert!(container.find_handler("second").is_some());
        assert!(container.find_handler("non-existent").is_none());
    }

    #[test]
    fn test_loading_duplicate_plugins() {
        let plugins: Vec<Arc<dyn MessagePlugin>> = vec![
            Arc::new(FirstPlugin {}),
            Arc::new(SecondPlugin {}),
            Arc::new(SecondPlugin {}),
        ];

        let mut container = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &plugins,
        };

        assert_eq!(
            container.load().unwrap_err(),
            MessageContainerError::DuplicateEntry
        );

        assert_eq!(container.collected_routes.len(), 0);
    }

    #[test]
    fn test_double_loading() {
        let plugins: Vec<Arc<dyn MessagePlugin>> =
            vec![Arc::new(FirstPlugin {}), Arc::new(SecondPlugin {})];

        let mut container = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &plugins,
        };

        assert!(container.load().is_ok());
        assert!(container.load().is_ok());
    }

    #[test]
    fn test_unloading_plugins() {
        let plugins: Vec<Arc<dyn MessagePlugin>> =
            vec![Arc::new(FirstPlugin {}), Arc::new(SecondPlugin {})];

        let mut container = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &plugins,
        };

        assert!(container.load().is_ok());
        assert_eq!(container.collected_routes.len(), 2);

        assert!(container.unload().is_ok());
        assert!(!container.loaded);

        assert_eq!(container.collected_routes.len(), 0);
    }

    #[test]
    fn test_routes_without_loading() {
        let plugins: Vec<Arc<dyn MessagePlugin>> =
            vec![Arc::new(FirstPlugin {}), Arc::new(SecondPlugin {})];

        let container = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &plugins,
        };

        // Attempt to access routes without loading
        assert_eq!(
            container.didcomm_routes().unwrap_err(),
            MessageContainerError::Unloaded
        );
    }
}
