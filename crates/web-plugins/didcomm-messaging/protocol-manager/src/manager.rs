use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};


use crate::api::{Plugin, PluginError, ProtocolRouter};

#[derive(Debug, PartialEq)]
pub enum ProtocolContainerError {
    DuplicateEntry,
    Unloaded,
    PluginErrorMap(HashMap<String, PluginError>),
}

pub struct ProtocolManager {
    plugins: Vec<Arc<Mutex<dyn Plugin>>>,
    message_type_handlers: ProtocolRouter,
    loaded: bool,
}

impl ProtocolManager {
    pub fn new() -> Self {
        ProtocolManager {
            plugins: Vec::new(),
            message_type_handlers: ProtocolRouter::new(),
            loaded: false,
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

    /// Load plugins and collect their message type handlers into the router.
    pub fn load(&mut self) -> Result<(), ProtocolContainerError> {
        tracing::debug!("loading plugin container");

        // Check for duplicate plugins
        let mut seen_names = HashSet::new();
        for plugin in &self.plugins {
            let plugin = plugin.lock().unwrap();
            if !seen_names.insert(plugin.name().to_string()) {
                tracing::error!("duplicate entry in plugin registry: {}", plugin.name());
                return Err(ProtocolContainerError::DuplicateEntry);
            }
        }

        // Reset the router before loading
        self.message_type_handlers = ProtocolRouter::new();

        // Try to mount plugins and populate the router with message type handlers
        let errors: HashMap<_, _> = self
            .plugins
            .iter()
            .filter_map(|plugin| {
                let mut plugin = plugin.lock().unwrap();
                match plugin.mount() {
                    Ok(_) => {
                        tracing::info!("mounted plugin {}", plugin.name());
                        // Insert each route into the `ProtocolRouter`
                        for (msg_type, handler) in plugin.routes().routes.into_iter() {
                            self.message_type_handlers
                                .routes(msg_type.as_str(), handler);
                        }
                        None
                    }
                    Err(err) => {
                        tracing::error!("error mounting plugin {}", plugin.name());
                        Some((plugin.name().to_string(), err))
                    }
                }
            })
            .collect();

        // Flag as loaded
        self.loaded = true;

        if errors.is_empty() {
            tracing::debug!("plugin container loaded successfully");
            Ok(())
        } else {
            Err(ProtocolContainerError::PluginErrorMap(errors))
        }
    }

    /// Unload plugins and clear the router's message type handlers.
    pub fn unload(&mut self) -> Result<(), ProtocolContainerError> {
        let mut errors = HashMap::new();

        for plugin in &self.plugins {
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
            self.message_type_handlers = ProtocolRouter::new(); 
            tracing::debug!("plugin container unloaded successfully");
            Ok(())
        } else {
            Err(ProtocolContainerError::PluginErrorMap(errors))
        }
    }

    /// Get the combined Axum router from all handlers if plugins are loaded.
    pub fn routes(&self) -> Result<ProtocolRouter, ProtocolContainerError> {
        if self.loaded {
            // Collect routes into an Axum Router
            Ok(self
                .message_type_handlers
                .routes
                .iter()
                .fold(ProtocolRouter::new(), |acc, (msg_type, handler)| {
                    acc.route(msg_type, axum::routing::get(handler.clone()))
                }))
        } else {
            Err(ProtocolContainerError::Unloaded)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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

        fn routes(&self) -> ProtocolRouter {
            let mut handlers = HashMap::new();
            handlers.insert(
                "first".to_string(),
                Box::new(|msg: &str| format!("Handled by first: {}", msg)) as Box<dyn Fn(&str) -> String + Send + Sync>,
            );
            ProtocolRouter { routes: handlers }
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

        fn routes(&self) -> ProtocolRouter {
            let mut handlers = HashMap::new();
            handlers.insert(
                "second".to_string(),
                Box::new(|msg: &str| format!("Handled by second: {}", msg)) as Box<dyn Fn(&str) -> String + Send + Sync>,
            );
            ProtocolRouter { routes: handlers }
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

        fn routes(&self) -> ProtocolRouter {
            ProtocolRouter::new()
        }
    }

    #[test]
    fn test_loading_plugins() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];

        let mut manager = ProtocolManager::new();
        manager.plugins = plugins;

        // Load plugins and validate success
        assert!(manager.load().is_ok());
        assert!(manager.loaded);

        // Verify routes
        let router_result = manager.routes();
        assert!(router_result.is_ok());
        let router = router_result.unwrap();
        assert!(router.routes.contains_key("first"));
        assert!(router.routes.contains_key("second"));
        assert_eq!(router.routes.len(), 2);
    }

    #[test]
    fn test_loading_with_faulty_plugin() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(FaultyPlugin {})),
        ];

        let mut manager = ProtocolManager::new();
        manager.plugins = plugins;

        let err = manager.load().unwrap_err();

        // Expected error map with faulty plugin initialization failure
        let mut expected_errors = HashMap::new();
        expected_errors.insert("faulty".to_string(), PluginError::InitError);

        assert_eq!(err, ProtocolContainerError::PluginErrorMap(expected_errors));
        assert_eq!(manager.message_type_handlers.routes.len(), 1); // Only FirstPlugin loaded
    }

    #[test]
    fn test_unloading_plugins() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];

        let mut manager = ProtocolManager::new();
        manager.plugins = plugins;

        // Load and confirm plugins are loaded
        assert!(manager.load().is_ok());
        assert!(manager.loaded);

        // Unload and confirm successful unloading
        assert!(manager.unload().is_ok());
        assert!(manager.message_type_handlers.routes.is_empty());
    }

    #[test]
    fn test_route_functionality() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];
    
        let mut manager = ProtocolManager::new();
        manager.plugins = plugins;
    
        // Ensure plugins are loaded
        assert!(manager.load().is_ok());
        assert!(manager.loaded); // Verify the manager is loaded
    
        // Validate router setup and functionality
        let router_result = manager.routes().unwrap();
        
        // Test first route
        let first_handler = router_result.routes.get("first").unwrap();
        assert_eq!(first_handler("test", "context"), "Handled by first: test context"); 
    
        // Test second route
        let second_handler = router_result.routes.get("second").unwrap();
        assert_eq!(second_handler("test", "context"), "Handled by second: test context"); 
    }
    
}