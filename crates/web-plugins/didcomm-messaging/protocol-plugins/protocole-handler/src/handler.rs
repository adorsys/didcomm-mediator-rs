use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use plugin_api::{Plugin, PluginError};

use crate::plugins::PLUGINS;

#[derive(Debug, PartialEq)]
pub enum PluginContainerError {
    DuplicateEntry,
    Unloaded,
    PluginErrorMap(HashMap<String, PluginError>),
}

pub struct PluginContainer<'a> {
    loaded: bool,
    message_type_handlers: HashMap<String, Box<dyn Fn(&str) -> String + Send + Sync>>,
    plugins: &'a Vec<Arc<Mutex<dyn Plugin>>>,
}

impl<'a> Default for PluginContainer<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PluginContainer<'a> {
    /// Instantiate an object aware of all statically registered plugins
    pub(crate) fn new() -> Self {
        Self {
            loaded: false,
            message_type_handlers: HashMap::new(),
            plugins: &PLUGINS,
        }
    }

    /// Search for a loaded plugin based on its name.
    pub fn find_plugin(&self, name: &str) -> Option<Arc<Mutex<dyn Plugin>>> {
        self.plugins.iter().find_map(|arc_plugin| {
            let plugin = arc_plugin.lock().unwrap();
            if plugin.name() == name {
                Some(Arc::clone(arc_plugin))
            } else {
                None
            }
        })
    }

    /// Load referenced plugins and collect their message type handlers internally.
    pub fn load(&mut self) -> Result<(), PluginContainerError> {
        tracing::debug!("loading plugin container");
    
        let mut seen_names = HashSet::new();
        let mut errors = HashMap::new();
    
        for plugin in self.plugins.iter() {
            let mut plugin_guard = plugin.lock().unwrap();
            let name = plugin_guard.name().to_string();
    
            // Check for duplicate names before mounting
            if seen_names.contains(&name) {
                tracing::error!("found duplicate entry in plugin registry: {}", name);
                return Err(PluginContainerError::DuplicateEntry);
            }
    
            if let Err(err) = plugin_guard.mount() {
                tracing::error!("error mounting plugin {}", name);
                errors.insert(name.clone(), err);
            } else {
                seen_names.insert(name.clone()); 
    
                tracing::info!("mounted plugin {}", name);
                self.message_type_handlers.insert(
                    format!("message.{}", name.clone()), 
                    Box::new(move |msg: &str| format!("Handled by {}: {}", name, msg)) as Box<dyn Fn(&str) -> String + Send + Sync>
                );
            }
        }
    
        self.loaded = true;
    
        if errors.is_empty() {
            tracing::debug!("plugin container loaded");
            Ok(())
        } else {
            Err(PluginContainerError::PluginErrorMap(errors))
        }
    }

    /// Unload container plugins and clear the HashMap of message type handlers.
    pub fn unload(&mut self) -> Result<(), PluginContainerError> {
        let mut errors = HashMap::new();

        for plugin in self.plugins.iter() {
            let plugin_guard = plugin.lock().unwrap();
            if let Err(err) = plugin_guard.unmount() {
                tracing::error!("error unmounting plugin {}", plugin_guard.name());
                errors.insert(plugin_guard.name().to_string(), err);
            } else {
                tracing::info!("unmounted plugin {}", plugin_guard.name());
            }
        }

        self.loaded = false;

        if errors.is_empty() {
            self.message_type_handlers.clear();
            tracing::debug!("plugin container unloaded");
            Ok(())
        } else {
            Err(PluginContainerError::PluginErrorMap(errors))
        }
    }

    /// Get the handler for a specific DIDComm message type.
    pub fn get_handler(&self, msg_type: &str) -> Result<&Box<dyn Fn(&str) -> String + Send + Sync>, PluginContainerError> {
        if !self.loaded {
            return Err(PluginContainerError::Unloaded);
        }
        
        self.message_type_handlers
            .get(msg_type)
            .ok_or(PluginContainerError::Unloaded)
    }
}

#[cfg(test)]
mod tests {
    use axum::routing::{Router, get};
    use super::*;
    use plugin_api::PluginError; 

    struct FirstPlugin;
    impl Plugin for FirstPlugin {
        fn name(&self) -> &'static str {
            "first"
        }

        fn mount(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        fn unmount(&self) -> Result<(), PluginError> {
            Ok(())
        }

        fn routes(&self) -> Router {
            Router::new().route("/first_route", get(|| async { "First Plugin Route" }))
        }
    }

    struct SecondPlugin;
    impl Plugin for SecondPlugin {
        fn name(&self) -> &'static str {
            "second"
        }

        fn mount(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        fn unmount(&self) -> Result<(), PluginError> {
            Ok(())
        }

        fn routes(&self) -> Router {
            Router::new().route("/second_route", get(|| async { "Second Plugin Route" }))
        }
    }

    struct FaultyPlugin;
    impl Plugin for FaultyPlugin {
        fn name(&self) -> &'static str {
            "faulty"
        }

        fn mount(&mut self) -> Result<(), PluginError> {
            Err(PluginError::InitError)
        }

        fn unmount(&self) -> Result<(), PluginError> {
            Ok(())
        }

        fn routes(&self) -> Router {
            Router::new().route("/Faulty_route", get(|| async { "Faulty Plugin Route" }))
        }
    }

    #[test]
    fn test_loading() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];

        let mut container = PluginContainer {
            loaded: false,
            message_type_handlers: HashMap::new(),
            plugins: &plugins,
        };

        // Test loading plugins
        assert!(container.load().is_ok());
        assert!(container.get_handler("message.first").is_ok());
        assert!(container.get_handler("message.second").is_ok());

        // Verify find_plugin method
        assert!(container.find_plugin("first").is_some());
        assert!(container.find_plugin("second").is_some());
        assert!(container.find_plugin("non-existent").is_none());

        assert_eq!(container.message_type_handlers.len(), 2);
    }
    
    #[test]
    fn test_loading_with_failing_plugin() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(FaultyPlugin {})),
        ];

        let mut container = PluginContainer {
            loaded: false,
            message_type_handlers: HashMap::new(),
            plugins: &plugins,
        };

        let err = container.load().unwrap_err();

        // Prepare expected error map
        let mut expected_error_map = HashMap::new();
        expected_error_map.insert("faulty".to_string(), PluginError::InitError);

        assert_eq!(err, PluginContainerError::PluginErrorMap(expected_error_map));

        // Verify collected message type handlers
        assert_eq!(container.message_type_handlers.len(), 1);
    }       

    #[test]
    fn test_unloading() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];

        let mut container = PluginContainer {
            loaded: false,
            message_type_handlers: HashMap::new(),
            plugins: &plugins,
        };

        // Load the plugins
        assert!(container.load().is_ok());
        assert!(container.loaded);

        // Now, we should unload the plugins
        let unload_result = container.unload();

        // Verify that unloading was successful
        assert!(unload_result.is_ok());
        assert!(container.message_type_handlers.is_empty());
    }

    #[test]
    fn test_get_handler() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];

        let mut container = PluginContainer {
            loaded: false,
            message_type_handlers: HashMap::new(),
            plugins: &plugins,
        };

        // Load the plugins
        container.load().unwrap();

        // Test the handler retrieval
        let first_handler = container.get_handler("message.first").unwrap();
        assert_eq!(first_handler("test"), "Handled by first: test");

        let second_handler = container.get_handler("message.second").unwrap();
        assert_eq!(second_handler("test"), "Handled by second: test");
    }
}
