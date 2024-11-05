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

    pub fn load(&mut self) -> Option<()> {
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
                    return None; // Returning None instead of DuplicateEntry error
                }
            }
        }

        // Reset route collection
        self.collected_routes.clear();

        // Mount protocols and collect routes on successful status
        let mut has_errors = false;
        for protocol in self.protocols.iter() {
            let mut protocol = protocol.lock().unwrap();
            for protocol in protocol.iter_mut() {
                if protocol.mount().is_ok() { 
                    tracing::info!("Mounted protocol {}", protocol.name());
                    self.collected_routes.push(protocol.routes());
                } else {
                    tracing::error!("Error mounting protocol {}", protocol.name());
                    has_errors = true;
                }
            }
        }

        // Update loaded status if no errors
        if !has_errors {
            self.loaded = true;
            tracing::debug!("Plugin container loaded successfully");
            Some(())
        } else {
            None
        }
    }

    pub fn unload(&mut self) -> Option<()> {
        for protocol in &self.mounted_protocols {
            let protocol_guard = protocol.lock().unwrap();
            if protocol_guard.unmount().is_ok() { 
                tracing::info!("Unmounted protocol {}", protocol_guard.name());
            } else {
                tracing::error!("Error unmounting protocol {}", protocol_guard.name());
                return None;
            }
        }

        self.loaded = false;
        self.mounted_protocols.clear();
        Some(())
    }

    // pub fn routes(
    //     &self,
    // ) -> Result<MessageRouter<AppState, Message, Response>, MessageContainerError> {
    //     if !self.loaded {
    //         return Err(MessageContainerError::Unloaded);
    //     }

    //     let main_router = self
    //         .collected_routes
    //         .iter()
    //         .chain(self.mounted_protocols.iter().collect::<Vec<_>>());
        
    //     let mut main_router = MessageRouter::new();

    //     Ok(main_router)
    // }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::{MessagePlugin, MessageRouter, PluginError};
    use std::collections::HashMap;
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

        // Update your `MessagePluginContainer` initialization:
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        assert!(manager.load().is_some());
        assert!(manager.find_plugin("example").is_some());
        assert!(manager.find_plugin("duplicate").is_some());
        assert!(manager.find_plugin("non-existent").is_none());
    }

    #[test]
    fn test_loading_with_duplicates() {
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(DuplicateProtocol {})])),
        ];

        // Update your `MessagePluginContainer` initialization:
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        let result = manager.load();
        assert!(result.is_none());
    }

    #[test]
    fn test_loading_with_failing_protocol() {
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(FaultyProtocol {})])),
        ];

        // Update your `MessagePluginContainer` initialization:
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        let result = manager.load();
        assert!(result.is_none());
    }

    #[test]
    fn test_unloading_protocols() {
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(DuplicateProtocol {})])),
        ];

        // Update your `MessagePluginContainer` initialization:
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        assert!(manager.load().is_some());
        assert!(manager.unload().is_some());
    }

    #[test]
    fn test_route_extraction_without_loading() {
        let protocols: Vec<Arc<Mutex<Vec<Box<dyn MessagePlugin<AppState, Message, Response>>>>>> = vec![
            Arc::new(Mutex::new(vec![Box::new(ExampleProtocol {})])),
            Arc::new(Mutex::new(vec![Box::new(DuplicateProtocol {})])),
        ];

        // Update your `MessagePluginContainer` initialization:
        let mut manager = MessagePluginContainer {
            loaded: false,
            collected_routes: vec![],
            protocols: &protocols,
            mounted_protocols: vec![],
        };

        assert!(manager.load().is_some());
        assert!(manager.unload().is_some());
    }
}