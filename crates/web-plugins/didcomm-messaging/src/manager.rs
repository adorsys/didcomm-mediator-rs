use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use axum::response::Response;
use didcomm::Message;
use shared::state::AppState;

use crate::protocols::{MessagePlugin, MessageRouter, PluginError, PROTOCOLS};

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
    protocols: &'a Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>>,
    mounted_protocols: Vec<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response> + 'static>>>,
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

    pub fn find_plugin(
        &self,
        name: &str,
    ) -> Option<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> {
        self.protocols.iter().find_map(|arc_protocol| {
            let protocol = arc_protocol.lock().unwrap();
            for plugin in protocol.iter() {
                if plugin.name() == name {
                    return Some(Arc::clone(arc_protocol));
                }
            }
            None
        })
    }

    pub fn load(&mut self) -> Result<(), MessageContainerError> {
        tracing::debug!("Loading plugin container");

        // Check for duplicates before mounting protocols
        let mut seen_names = HashSet::new();
        for protocol in self.protocols.iter() {
            let protocol = protocol.lock().unwrap();
            for protocol in protocol.iter() {
                if !seen_names.insert(protocol.name().to_string()) {
                    tracing::error!(
                        "Found duplicate entry in protocols registry: {}",
                        protocol.name()
                    );
                    return Err(MessageContainerError::DuplicateEntry);
                }
            }
        }

        // Reset route collection
        self.collected_routes.clear();

        // Mount protocols and collect routes on successful status
        let errors: HashMap<_, _> = self
            .protocols
            .iter()
            .filter_map(|protocol| {
                let mut protocol = protocol.lock().unwrap();
                let mut error_map = HashMap::new();
                for protocol in protocol.iter_mut() {
                    match protocol.mount() {
                        Ok(_) => {
                            tracing::info!("Mounted protocol {}", protocol.name());
                            self.collected_routes.push(protocol.routes());
                        }
                        Err(err) => {
                            tracing::error!("Error mounting protocol {}", protocol.name());
                            error_map.insert(protocol.name().to_string(), err);
                        }
                    }
                }
                if error_map.is_empty() {
                    None
                } else {
                    Some(error_map)
                }
            })
            .flatten()
            .collect();

        // Update loaded status
        self.loaded = true;

        // Return load status
        if errors.is_empty() {
            tracing::debug!("Plugin container loaded successfully");
            Ok(())
        } else {
            Err(MessageContainerError::ProtocolErrorMap(errors))
        }
    }

    pub fn unload(&mut self) -> Result<(), MessageContainerError> {
        let mut errors = HashMap::new();

        for protocol in &self.mounted_protocols {
            let protocol = protocol.lock().unwrap();
            if let Err(err) = protocol.unmount() {
                tracing::error!("Error unmounting protocol {}", protocol.name());
                errors.insert(protocol.name().to_string(), err);
            } else {
                tracing::info!("Unmounted protocol {}", protocol.name());
            }
        }

        if !errors.is_empty() {
            return Err(MessageContainerError::ProtocolErrorMap(errors));
        }

        self.loaded = false;
        self.mounted_protocols.clear();
        Ok(())
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
    
        // Add routes from mounted protocols after converting them
        for protocol in &self.mounted_protocols {
            let protocol = protocol.lock().unwrap();
            main_router.merge(&protocol.routes());
        }
    
        Ok(main_router)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::{MessagePlugin, MessageRouter, PluginError};
    use std::sync::{Arc, Mutex};

    struct ExampleProtocol;
    impl<S, M, R> MessagePlugin<S, M, R> for ExampleProtocol
    where
        S: Clone + Sync + Send + 'static,
        M: Send + 'static,
        R: Send + 'static,
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

        fn routes(&self) -> MessageRouter<S, M, R> {
            MessageRouter::new()
        }
    }

    struct DuplicateProtocol;
    impl<S, M, R> MessagePlugin<S, M, R> for DuplicateProtocol
    where
        S: Clone + Sync + Send + 'static,
        M: Send + 'static,
        R: Send + 'static,
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

        fn routes(&self) -> MessageRouter<S, M, R> {
            MessageRouter::new()
        }
    }

    struct FaultyProtocol;
    impl<S, M, R> MessagePlugin<S, M, R> for FaultyProtocol
    where
        S: Clone + Sync + Send + 'static,
        M: Send + 'static,
        R: Send + 'static,
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

        fn routes(&self) -> MessageRouter<S, M, R> {
            MessageRouter::new()
        }
    }

    #[test]
    fn test_loading_protocols() {
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(DuplicateProtocol {})])),
        ];

        // Create MessagePluginContainer
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        assert!(manager.load().is_ok());
        assert!(manager.find_plugin("example").is_some());
        assert!(manager.find_plugin("duplicate").is_some());
        assert!(manager.find_plugin("non-existent").is_none());
    }

    #[test]
    fn test_loading_with_duplicates() {
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(DuplicateProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(DuplicateProtocol {})])), 
        ];
    
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };
    
        let result = manager.load();
        assert!(result.is_err(), "Expected loading with duplicates to return an error");
    }
    

    #[test]
    fn test_loading_with_failing_protocol() {
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(FaultyProtocol {})])),
        ];

        // Initialize the MessagePluginContainer with protocols
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        let result = manager.load();
        assert!(result.is_err(), "Expected loading with a faulty protocol to return an error");
    }

    #[test]
    fn test_unloading_protocols() {
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(DuplicateProtocol {})])),
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
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(DuplicateProtocol {})])),
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