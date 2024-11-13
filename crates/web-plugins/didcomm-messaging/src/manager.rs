use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use axum::response::Response;
use didcomm::{protocols, Message};
use shared::state::AppState;

use crate::protocol::PROTOCOLS;
use message_api::{MessagePlugin, MessageRouter, PluginError};

type DidcommRouter = MessageRouter<AppState, Message, Response>;

#[derive(Debug, PartialEq)]
pub enum MessageContainerError {
    DuplicateEntry,
    Unloaded,
    ProtocolErrorMap(HashMap<String, PluginError>),
}

pub struct MessagePluginContainer<'a> {
    loaded: bool,
    collected_routes: Vec<DidcommRouter>,
    protocols: &'a Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>>,
    mounted_protocols: Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>>,
}

impl<'a> MessagePluginContainer<'a> {
    pub fn new() -> Self {
        Self {
            loaded: false,
            collected_routes: vec![],
            protocols: &PROTOCOLS,
            mounted_protocols: vec![],
        }
    }

    pub fn find_protocol(
        &self,
        name: &str,
    ) -> Option<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>> {
        self.protocols.iter().find_map(|arc_protocol| {
            let protocol = arc_protocol.lock().unwrap();
            (protocol.name() == name).then_some(Arc::clone(&arc_protocol))
        })
    }

    pub fn load(&mut self) -> Result<(), MessageContainerError> {
        tracing::debug!("Loading protocol container");

        let mut seen_names = HashSet::new();
        for protocol in self.protocols.iter() {
            let protocol = protocol.lock().unwrap();
            if !seen_names.insert(protocol.name().to_string()) {
                tracing::error!(
                    "found duplicate entry in protocol registry: {}",
                    protocol.name()
                );
                return Err(MessageContainerError::DuplicateEntry);
            }
        }

        // Reset route collection
        self.collected_routes.clear();

        // Mount protocols and collect routes on successful status
        let errors: HashMap<_, _> = self
            .protocols
            .iter()
            .filter_map(|protocol| {
                let protocol_clone = protocol.clone();
                let mut protocol = protocol.lock().unwrap();
                match protocol.mount() {
                    Ok(_) => {
                        tracing::info!("mounted protocol {}", protocol.name());
                        self.collected_routes.push(protocol.routes());
                        self.mounted_protocols.push(protocol_clone);
                        None
                    }
                    Err(err) => {
                        tracing::error!("error mounting protocol {}", protocol.name());
                        Some((protocol.name().to_string(), err))
                    }
                }
            })
            .collect();

        // Update loaded status
        self.loaded = true;

        // Return load status
        if errors.is_empty() {
            tracing::debug!("protocol container loaded successfully");
            Ok(())
        } else {
            Err(MessageContainerError::ProtocolErrorMap(errors))
        }
    }

    /// unload container protocols
    pub fn unload(&mut self) -> Result<(), MessageContainerError> {
        // Unmount protocols and clearing the vector of routes
        let errors: HashMap<_, _> = self
            .mounted_protocols
            .iter()
            .filter_map(|protocol| {
                let protocol = protocol.lock().unwrap();
                match protocol.unmount() {
                    Ok(_) => {
                        tracing::info!("unmounted protocol {}", protocol.name());
                        None
                    }
                    Err(err) => {
                        tracing::error!("error unmounting protocol {}", protocol.name());
                        Some((protocol.name().to_owned(), err))
                    }
                }
            })
            .collect();

        // Flag as unloaded
        self.loaded = false;

        // Return state of completion
        if errors.is_empty() {
            self.collected_routes.clear();
            tracing::debug!("protocol container unloaded");
            Ok(())
        } else {
            Err(MessageContainerError::ProtocolErrorMap(errors))
        }
    }

    pub fn routes(
        &self,
    ) -> Result<MessageRouter<AppState, Message, Response>, MessageContainerError> {
        if !self.loaded {
            return Err(MessageContainerError::Unloaded);
        }

        let mut main_router = MessageRouter::new();

        // Add routes from collected routes
        for route in &self.collected_routes {
            main_router.merge(route);
        }

        Ok(main_router)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use message_api::{MessagePlugin, MessageRouter, PluginError};
    use std::sync::{Arc, Mutex};

    struct ExampleProtocol;
    impl<S, M, E> MessagePlugin<S, M, E> for ExampleProtocol
    where
        S: Clone + Sync + Send + 'static,
        M: Send + 'static,
        E: Send + 'static,
    {
        fn name(&self) -> &'static str {
            "example"
        }

        fn mount(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        fn unmount(&self) -> Result<(), PluginError> {
            Ok(())
        }

        fn routes(&self) -> MessageRouter<S, M, E> {
            MessageRouter::new()
        }
    }

    struct DuplicateProtocol;
    impl<S, M, E> MessagePlugin<S, M, E> for DuplicateProtocol
    where
        S: Clone + Sync + Send + 'static,
        M: Send + 'static,
        E: Send + 'static,
    {
        fn name(&self) -> &'static str {
            "duplicate"
        }

        fn mount(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        fn unmount(&self) -> Result<(), PluginError> {
            Ok(())
        }

        fn routes(&self) -> MessageRouter<S, M, E> {
            MessageRouter::new()
        }
    }

    struct FaultyProtocol;
    impl<S, M, E> MessagePlugin<S, M, E> for FaultyProtocol
    where
        S: Clone + Sync + Send + 'static,
        M: Send + 'static,
        E: Send + 'static,
    {
        fn name(&self) -> &'static str {
            "faulty"
        }

        fn mount(&mut self) -> Result<(), PluginError> {
            Err(PluginError::InitError)
        }

        fn unmount(&self) -> Result<(), PluginError> {
            Ok(())
        }

        fn routes(&self) -> MessageRouter<S, M, E> {
            MessageRouter::new()
        }
    }

    #[test]
    fn test_loading_protocols() {
        let protocols: Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>> = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
        ];

        // Create MessagePluginContainer
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        assert!(manager.load().is_ok());
        assert!(manager.find_protocol("example").is_some());
        assert!(manager.find_protocol("duplicate").is_some());
        assert!(manager.find_protocol("non-existent").is_none());
    }

    #[test]
    fn test_loading_with_duplicates() {
        let protocols: Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>> = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
        ];

        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        let result = manager.load();
        assert!(
            result.is_err(),
            "Expected loading with duplicates to return an error"
        );
    }

    #[test]
    fn test_loading_with_failing_protocol() {
        let protocols: Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>> = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(FaultyProtocol {})),
        ];

        // Initialize the MessagePluginContainer with protocols
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        let result = manager.load();
        assert!(
            result.is_err(),
            "Expected loading with a faulty protocol to return an error"
        );
    }

    #[test]
    fn test_unloading_protocols() {
        let protocols: Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>> = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
        ];

        // Create MessagePluginContainer
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        // Load the protocols and assert it's successful
        assert!(manager.load().is_ok());
        assert!(manager.unload().is_ok());
    }

    #[test]
    fn test_route_extraction() {
        let protocols: Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>> = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
        ];

        // Initialize MessagePluginContainer
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        // Load and unload protocols to test route extraction
        assert!(manager.load().is_ok());
        assert!(manager.unload().is_ok());
    }
}
