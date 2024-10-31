use crate::protocols::{Plugin, PluginError, ProtocolRouter};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Debug, PartialEq)]
pub enum ProtocolContainerError {
    DuplicateEntry,
    Unloaded,
    ProtocolErrorMap(HashMap<String, PluginError>),
}

pub struct ProtocolManager<F, T, M, R>
where
    M: serde::Serialize + for<'de> serde::Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
    protocols: Vec<Arc<Mutex<dyn Plugin<F, T, M, R>>>>,
    message_type_handlers: ProtocolRouter<F, T, M, R>,
    loaded: bool,
}

impl<F, T, M, R> ProtocolManager<F, T, M, R>
where
    M: serde::Serialize + for<'de> serde::Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
    pub fn new() -> Self {
        ProtocolManager {
            protocols: Vec::new(),
            message_type_handlers: ProtocolRouter::new(),
            loaded: false,
        }
    }

    pub fn find_plugin(&self, name: &str) -> Option<Arc<Mutex<dyn Plugin<F, T, M, R>>>> {
        self.protocols.iter().find_map(|arc_protocol| {
            arc_protocol.lock().ok().and_then(|protocol| {
                if protocol.name().trim() == name.trim() {
                    Some(Arc::clone(arc_protocol))
                } else {
                    None
                }
            })
        })
    }

    pub fn load(&mut self) -> Result<(), ProtocolContainerError> {
        tracing::debug!("loading plugin container");

        let mut seen_names = HashSet::new();
        for protocol in self.protocols.iter().cloned() {
            let protocol = protocol
                .lock()
                .map_err(|_| ProtocolContainerError::Unloaded)?;
            if !seen_names.insert(protocol.name().to_string()) {
                tracing::error!("duplicate entry in protocol registry: {}", protocol.name());
                return Err(ProtocolContainerError::DuplicateEntry);
            }
        }

        self.message_type_handlers = ProtocolRouter::new();

        let errors: HashMap<_, _> = self
            .protocols
            .iter()
            .filter_map(|protocol| {
                let mut protocol = protocol.lock().ok()?;
                match protocol.mount() {
                    Ok(_) => {
                        tracing::info!("mounted protocol {}", protocol.name());
                        let protocol_routes = protocol.get_routes();
                        for (msg_type, handler) in protocol_routes.route_map {
                            self.message_type_handlers =
                                std::mem::take(&mut self.message_type_handlers)
                                    .add_route(&msg_type, handler);
                        }
                        self.loaded = true;
                        None
                    }
                    Err(err) => {
                        tracing::error!("error mounting protocol {}", protocol.name());
                        Some((protocol.name().to_string(), err))
                    }
                }
            })
            .collect();

        if errors.is_empty() {
            tracing::debug!("protocol container loaded successfully");
            Ok(())
        } else {
            Err(ProtocolContainerError::ProtocolErrorMap(errors))
        }
    }

    pub fn unload(&mut self) -> Result<(), ProtocolContainerError> {
        let mut errors = HashMap::new();

        for protocol in &self.protocols {
            let protocol_guard = protocol
                .lock()
                .map_err(|_| ProtocolContainerError::Unloaded)?;
            if let Err(err) = protocol_guard.unmount() {
                tracing::error!("error unmounting protocol {}", protocol_guard.name());
                errors.insert(protocol_guard.name().to_string(), err);
            } else {
                tracing::info!("unmounted protocol {}", protocol_guard.name());
            }
        }

        self.loaded = false;
        self.message_type_handlers = ProtocolRouter::new();

        if errors.is_empty() {
            tracing::debug!("protocol container unloaded successfully");
            Ok(())
        } else {
            Err(ProtocolContainerError::ProtocolErrorMap(errors))
        }
    }

    pub fn routes(
        protocols: Vec<ProtocolRouter<F, T, M, R>>,
    ) -> Result<Self, ProtocolContainerError> {
        if protocols.is_empty() {
            return Err(ProtocolContainerError::Unloaded);
        }

        let mut main_router = ProtocolRouter::new();

        for protocol in protocols {
            main_router = main_router.merge(protocol);
        }

        Ok(ProtocolManager {
            protocols: Vec::new(),
            message_type_handlers: main_router,
            loaded: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::{Plugin, PluginError, ProtocolRouter};
    use std::sync::{Arc, Mutex};

    struct ExampleProtocol;
    impl<F, T, M, R> Plugin<F, T, M, R> for ExampleProtocol
    where
        M: serde::Serialize + for<'de> serde::Deserialize<'de>,
        F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
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

        fn get_routes(&self) -> ProtocolRouter<F, T, M, R> {
            ProtocolRouter::new()
        }
    }

    struct DuplicateProtocol;
    impl<F, T, M, R> Plugin<F, T, M, R> for DuplicateProtocol
    where
        M: serde::Serialize + for<'de> serde::Deserialize<'de>,
        F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
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

        fn get_routes(&self) -> ProtocolRouter<F, T, M, R> {
            ProtocolRouter::new()
        }
    }

    struct FaultyProtocol;
    impl<F, T, M, R> Plugin<F, T, M, R> for FaultyProtocol
    where
        M: serde::Serialize + for<'de> serde::Deserialize<'de>,
        F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
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

        fn get_routes(&self) -> ProtocolRouter<F, T, M, R> {
            ProtocolRouter::new()
        }
    }

    #[test]
    fn test_loading_protocols() {
        let protocols: Vec<
            Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
        > = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
        ];

        let mut manager = ProtocolManager {
            protocols,
            message_type_handlers: ProtocolRouter::new(),
            loaded: false,
        };

        assert!(manager.load().is_ok());
        assert!(manager.find_plugin("example").is_some());
        assert!(manager.find_plugin("duplicate").is_some());
        assert!(manager.find_plugin("non-existent").is_none());
    }

    #[test]
    fn test_loading_with_duplicates() {
        let protocols: Vec<
            Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
        > = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
        ];

        let mut manager = ProtocolManager {
            protocols,
            message_type_handlers: ProtocolRouter::new(),
            loaded: false,
        };

        let result = manager.load();
        assert_eq!(result.unwrap_err(), ProtocolContainerError::DuplicateEntry);
    }

    #[test]
    fn test_loading_with_failing_protocol() {
        let protocols: Vec<
            Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
        > = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(FaultyProtocol {})),
        ];

        let mut manager = ProtocolManager {
            protocols,
            message_type_handlers: ProtocolRouter::new(),
            loaded: false,
        };

        let err = manager.load().unwrap_err();
        let mut expected_error_map = HashMap::new();
        expected_error_map.insert("faulty".to_string(), PluginError::InitError);

        assert_eq!(
            err,
            ProtocolContainerError::ProtocolErrorMap(expected_error_map)
        );
    }
    #[test]
    fn test_unloading_protocols() {
        let protocols: Vec<
            Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
        > = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
        ];

        let mut manager = ProtocolManager {
            protocols,
            message_type_handlers: ProtocolRouter::new(),
            loaded: false,
        };

        assert!(manager.load().is_ok());
        assert!(manager.unload().is_ok());
    }
    #[test]
    fn test_route_extraction_without_loading() {
        let protocols: Vec<
            Arc<Mutex<dyn Plugin<fn(&(), &String) -> Result<String, ()>, (), String, ()>>>,
        > = vec![
            Arc::new(Mutex::new(ExampleProtocol {})),
            Arc::new(Mutex::new(DuplicateProtocol {})),
        ];

        let mut manager = ProtocolManager {
            protocols,
            message_type_handlers: ProtocolRouter::new(),
            loaded: false,
        };

        assert!(manager.load().is_ok());
        assert!(manager.unload().is_ok());
    }
}
