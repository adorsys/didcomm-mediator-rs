use crate::protocols::DIDCOMM_PLUGINS;
use message_api::{MessagePlugin, MessageRouter};
use std::{collections::HashSet, sync::Arc};

#[derive(Debug, PartialEq)]
pub(crate) enum MessageContainerError {
    DuplicateEntry,
    Unloaded,
}

pub(crate) struct MessagePluginContainer<'a> {
    pub(crate) loaded: bool,
    pub(crate) collected_routes: Vec<MessageRouter>,
    pub(crate) message_plugins: &'a Vec<Arc<dyn MessagePlugin>>,
}

impl<'a> MessagePluginContainer<'a> {
    pub(crate) fn new() -> Self {
        Self {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &DIDCOMM_PLUGINS,
        }
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
    fn test_routes_without_loading() {
        let plugins: Vec<Arc<dyn MessagePlugin>> =
            vec![Arc::new(FirstPlugin {}), Arc::new(SecondPlugin {})];

        let container = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            message_plugins: &plugins,
        };

        // Attempt to access routes without loading
        assert!(
            container.didcomm_routes().is_err(),
            "Routes should not be accessible without loading"
        );

        if let Err(err) = container.didcomm_routes() {
            assert_eq!(err, MessageContainerError::Unloaded);
        }
    }
}
