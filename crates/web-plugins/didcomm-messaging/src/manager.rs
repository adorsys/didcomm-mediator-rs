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
    protocols: &'a [Arc<Mutex<dyn MessagePlugin<AppState, Message, Response> + 'static>>],
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

    pub fn find_plugin(&self, name: &str) -> Option<Arc<Mutex<dyn MessagePlugin<AppState, Message, Response>>>> {
        self.protocols.iter().find_map(|arc_protocol| {
            let protocol = arc_protocol.lock().unwrap();
            if protocol.name() == name {
                Some(Arc::clone(arc_protocol))
            } else {
                None
            }
        })
    }

    pub fn load(&mut self) -> Result<(), MessageContainerError> {
        tracing::debug!("Loading plugin container");

        // Check for duplicates before mounting protocols
        let mut seen_names = HashSet::new();
        for protocol in self.protocols.iter() {
            let protocol = protocol.lock().unwrap();
            if !seen_names.insert(protocol.name().to_string()) {
                tracing::error!("Found duplicate entry in protocols registry: {}", protocol.name());
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
                let mut protocol = protocol.lock().unwrap();
                match protocol.mount() {
                    Ok(_) => {
                        tracing::info!("Mounted protocol {}", protocol.name());
                        self.collected_routes.push(protocol.routes());
                        None
                    }
                    Err(err) => {
                        tracing::error!("Error mounting protocol {}", protocol.name());
                        Some((protocol.name().to_string(), err))
                    }
                }
            })
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
            let protocol_guard = protocol.lock().unwrap();
            if let Err(err) = protocol_guard.unmount() {
                tracing::error!("Error unmounting protocol {}", protocol_guard.name());
                errors.insert(protocol_guard.name().to_string(), err);
            } else {
                tracing::info!("Unmounted protocol {}", protocol_guard.name());
            }
        }

        if !errors.is_empty() {
            return Err(MessageContainerError::ProtocolErrorMap(errors));
        }

        self.loaded = false;
        self.mounted_protocols.clear();
        Ok(())
    }
    
    pub fn routes(&self) -> Result<MessageRouter<AppState, Message, Response>, MessageContainerError> {
        if !self.loaded() {
            return Err(MessageContainerError::Unloaded);
        }

        let main_router = self.collected_routes.iter().chain(self.mounted_protocols.iter().collect());
        
        let mut main_router = MessageRouter::new();
        
        
        Ok(main_router)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::protocols::{MessageRouter, Plugin, PluginError};
//     use std::sync::{Arc, Mutex};

//     struct ExampleProtocol;
//     impl<F, T, M, R> Plugin<F, T, M, R> for ExampleProtocol
//     where
//         M: serde::Serialize + for<'de> serde::Deserialize<'de>,
//         F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
//     {
//         fn name(&self) -> &'static str {
//             "example"
//         }

//         fn mount(&mut self) -> Result<(), PluginError> {
//             Ok(())
//         }

//         fn unmount(&self) -> Result<(), PluginError> {
//             Ok(())
//         }

//         fn get_routes(&self) -> MessageRouter<F, T, M, R> {
//             MessageRouter::new()
//         }
//     }

//     struct DuplicateProtocol;
//     impl<F, T, M, R> Plugin<F, T, M, R> for DuplicateProtocol
//     where
//         M: serde::Serialize + for<'de> serde::Deserialize<'de>,
//         F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
//     {
//         fn name(&self) -> &'static str {
//             "duplicate"
//         }

//         fn mount(&mut self) -> Result<(), PluginError> {
//             Ok(())
//         }

//         fn unmount(&self) -> Result<(), PluginError> {
//             Ok(())
//         }

//         fn get_routes(&self) -> MessageRouter<F, T, M, R> {
//             MessageRouter::new()
//         }
//     }

//     struct FaultyProtocol;
//     impl<F, T, M, R> Plugin<F, T, M, R> for FaultyProtocol
//     where
//         M: serde::Serialize + for<'de> serde::Deserialize<'de>,
//         F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
//     {
//         fn name(&self) -> &'static str {
//             "faulty"
//         }

//         fn mount(&mut self) -> Result<(), PluginError> {
//             Err(PluginError::InitError)
//         }

//         fn unmount(&self) -> Result<(), PluginError> {
//             Ok(())
//         }

//         fn get_routes(&self) -> MessageRouter<F, T, M, R> {
//             MessageRouter::new()
//         }
//     }

//     #[test]
//     fn test_loading_protocols() {
//         let protocols: Vec<
//             Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
//         > = vec![
//             Arc::new(Mutex::new(ExampleProtocol {})),
//             Arc::new(Mutex::new(DuplicateProtocol {})),
//         ];

//         let mut manager = ProtocolManager {
//             protocols,
//             message_type_handlers: MessageRouter::new(),
//             loaded: false,
//         };

//         assert!(manager.load().is_ok());
//         assert!(manager.find_plugin("example").is_some());
//         assert!(manager.find_plugin("duplicate").is_some());
//         assert!(manager.find_plugin("non-existent").is_none());
//     }

//     #[test]
//     fn test_loading_with_duplicates() {
//         let protocols: Vec<
//             Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
//         > = vec![
//             Arc::new(Mutex::new(ExampleProtocol {})),
//             Arc::new(Mutex::new(DuplicateProtocol {})),
//             Arc::new(Mutex::new(DuplicateProtocol {})),
//         ];

//         let mut manager = ProtocolManager {
//             protocols,
//             message_type_handlers: MessageRouter::new(),
//             loaded: false,
//         };

//         let result = manager.load();
//         assert_eq!(result.unwrap_err(), MessageContainerError::DuplicateEntry);
//     }

//     #[test]
//     fn test_loading_with_failing_protocol() {
//         let protocols: Vec<
//             Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
//         > = vec![
//             Arc::new(Mutex::new(ExampleProtocol {})),
//             Arc::new(Mutex::new(FaultyProtocol {})),
//         ];

//         let mut manager = ProtocolManager {
//             protocols,
//             message_type_handlers: MessageRouter::new(),
//             loaded: false,
//         };

//         let err = manager.load().unwrap_err();
//         let mut expected_error_map = HashMap::new();
//         expected_error_map.insert("faulty".to_string(), PluginError::InitError);

//         assert_eq!(
//             err,
//             MessageContainerError::ProtocolErrorMap(expected_error_map)
//         );
//     }
//     #[test]
//     fn test_unloading_protocols() {
//         let protocols: Vec<
//             Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
//         > = vec![
//             Arc::new(Mutex::new(ExampleProtocol {})),
//             Arc::new(Mutex::new(DuplicateProtocol {})),
//         ];

//         let mut manager = ProtocolManager {
//             protocols,
//             message_type_handlers: MessageRouter::new(),
//             loaded: false,
//         };

//         assert!(manager.load().is_ok());
//         assert!(manager.unload().is_ok());
//     }
//     #[test]
//     fn test_route_extraction_without_loading() {
//         let protocols: Vec<
//             Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
//         > = vec![
//             Arc::new(Mutex::new(ExampleProtocol {})),
//             Arc::new(Mutex::new(DuplicateProtocol {})),
//         ];

//         let mut manager = ProtocolManager {
//             protocols,
//             message_type_handlers: MessageRouter::new(),
//             loaded: false,
//         };

//         assert!(manager.load().is_ok());
//         assert!(manager.unload().is_ok());
//     }
// }
