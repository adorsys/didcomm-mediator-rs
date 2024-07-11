use axum::Router;
use server_plugin::{Plugin, PluginError};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use super::PLUGINS;

#[derive(Debug, PartialEq)]
pub enum PluginContainerError {
    DuplicateEntry,
    Unloaded,
    PluginErrorMap(HashMap<String, PluginError>),
}
pub struct PluginContainer {
    loaded: bool,
    collected_routes: Vec<Router>,
    plugins: Arc<Vec<Arc<Mutex<dyn Plugin>>>>,
}

impl Default for PluginContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginContainer {
    /// Instantiate an object aware of all statically registered plugins
    pub fn new() -> Self {
        Self {
            loaded: false,
            collected_routes: vec![],
            plugins: Arc::clone(&PLUGINS),
        }
    }

    /// Search loaded plugin based on name string
    pub fn find_plugin(&self, name: &str) -> Option<Arc<Mutex<dyn Plugin>>> {
        self.plugins.iter().find_map(|arc_plugin| {
            let plugin = arc_plugin.lock().unwrap();
            (plugin.name() == name).then_some(Arc::clone(&arc_plugin))
        })
    }

    /// Load referenced plugins
    ///
    /// This entails mounting them and merging their routes internally (only
    /// upon successful initialization). An error is returned if plugins
    /// bearing the same name are found. Also, all plugins failing to be
    /// initialized are returned in a map with respectively raised errors.
    pub fn load(&mut self) -> Result<(), PluginContainerError> {
        tracing::debug!("loading plugin container");

        // Checking for duplicates before mounting plugins
        let mut seen_names = HashSet::new();
        for plugin in self.plugins.iter() {
            let plugin = plugin.lock().unwrap();
            if !seen_names.insert(plugin.name().to_string()) {
                tracing::error!(
                    "found duplicate entry in plugin registry: {}",
                    plugin.name()
                );
                return Err(PluginContainerError::DuplicateEntry);
            }
        }

        // Reset collection of routes
        self.collected_routes.clear();

        // Mount plugins and collect routes on successful status
        let errors: HashMap<_, _> = self
            .plugins
            .iter()
            .filter_map(|plugin| {
                let mut plugin = plugin.lock().unwrap();
                match plugin.mount() {
                    Ok(_) => {
                        tracing::info!("mounted plugin {}", plugin.name());
                        self.collected_routes.push(plugin.routes());
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

        // Return state of completion
        if errors.is_empty() {
            tracing::debug!("plugin container loaded");
            Ok(())
        } else {
            Err(PluginContainerError::PluginErrorMap(errors))
        }
    }

    /// unload container plugins
    pub fn unload(&mut self) -> Result<(), PluginContainerError> {
        // Unmount plugins and clearing the vector of routes
        let errors: HashMap<_, _> = self
            .plugins
            .iter()
            .filter_map(|plugin| {
                let plugin = plugin.lock().unwrap();
                match plugin.unmount() {
                    Ok(_) => {
                        tracing::info!("Unmounted plugin {}", plugin.name());
                        None
                    }
                    Err(err) => {
                        tracing::error!("error unmounting plugin {}", plugin.name());
                        Some((plugin.name().to_owned(), err))
                    }
                }
            })
            .collect();

        // Flag as unloaded
        self.loaded = false;

        // Return state of completion
        if errors.is_empty() {
            self.collected_routes.clear();
            tracing::debug!("plugin container unloaded");
            Ok(())
        } else {
            Err(PluginContainerError::PluginErrorMap(errors))
        }
    }

    /// Merge collected routes from all plugins successfully initialized.
    pub fn routes(&self) -> Result<Router, PluginContainerError> {
        if self.loaded {
            Ok(self
                .collected_routes
                .iter()
                .fold(Router::new(), |acc, e| acc.merge(e.clone())))
        } else {
            Err(PluginContainerError::Unloaded)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;

    // Define plugin structs for testing
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
            Router::new().route("/first", get(|| async {}))
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
            Router::new().route("/second", get(|| async {}))
        }
    }

    struct SecondAgainPlugin;
    impl Plugin for SecondAgainPlugin {
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
            Router::new().route("/second", get(|| async {}))
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
            Router::new().route("/faulty", get(|| async {}))
        }
    }

    #[test]
    fn test_loading() {
        // Mock plugins for testing
        let plugins: Arc<Vec<Arc<Mutex<dyn Plugin>>>> = Arc::new(vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ]);

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: Arc::clone(&plugins),
        };

        // Test loading plugins
        assert!(container.load().is_ok());
        assert!(container.routes().is_ok());

        // Verify find_plugin method
        assert!(container.find_plugin("first").is_some());
        assert!(container.find_plugin("second").is_some());
        assert!(container.find_plugin("non-existent").is_none());

        // Verify collected routes
        // The actual routes collected are actually hard to test
        // given that axum::Router seems not to provide public
        // directives to inquire internal state.
        // See: https://github.com/tokio-rs/axum/discussions/860
        assert_eq!(container.collected_routes.len(), 2);
    }

    #[test]
    fn test_double_loading() {
        // Mock plugins for testing
        let plugins: Arc<Vec<Arc<Mutex<dyn Plugin>>>> = Arc::new(vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ]);

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: Arc::clone(&plugins),
        };

        // Test loading plugins twice
        assert!(container.load().is_ok());
        assert!(container.load().is_ok()); // Load again, should succeed without errors

        // Verify collected routes
        assert_eq!(container.collected_routes.len(), 2);
    }

    #[test]
    fn test_loading_with_duplicates() {
        let plugins: Arc<Vec<Arc<Mutex<dyn Plugin>>>> = Arc::new(vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
            Arc::new(Mutex::new(SecondAgainPlugin {})),
        ]);

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: Arc::clone(&plugins),
        };

        // Attempt to load plugins with duplicates
        let result = container.load();

        // Assert that the result is an error due to duplicate entries
        assert_eq!(result.unwrap_err(), PluginContainerError::DuplicateEntry);

        // Verify collected routes (should not be affected by duplicates)
        assert_eq!(container.collected_routes.len(), 0); // No routes should be collected on error
    }

    #[test]
    fn test_loading_with_failing_plugin() {
        // Mock plugins for testing
        let plugins: Arc<Vec<Arc<Mutex<dyn Plugin>>>> = Arc::new(vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(FaultyPlugin {})),
        ]);

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: Arc::clone(&plugins),
        };

        let err = container.load().unwrap_err();

        // Prepare expected error map
        let mut expected_error_map = HashMap::new();
        expected_error_map.insert("faulty".to_string(), PluginError::InitError);

        assert_eq!(
            err,
            PluginContainerError::PluginErrorMap(expected_error_map)
        );

        // Verify collected routes
        assert_eq!(container.collected_routes.len(), 1);
    }

    #[test]
    fn test_route_extraction_without_loading() {
        // Mock plugins for testing
        let plugins: Arc<Vec<Arc<Mutex<dyn Plugin>>>> = Arc::new(vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ]);

        // Initialize PluginContainer with the mock plugins
        let container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: Arc::clone(&plugins),
        };

        // Test route extraction without loading
        assert_eq!(
            container.routes().unwrap_err(),
            PluginContainerError::Unloaded
        );
    }
    #[test]
    fn test_unloading() {
        // loading container from test_loading
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: Arc::new(vec![
                Arc::new(Mutex::new(FirstPlugin {})),
                Arc::new(Mutex::new(SecondPlugin {})),
            ]),
        };
        assert!(container.load().is_ok());
        assert!(container.routes().is_ok());

        // unloading container and clearing routes
        assert!(container.unload().is_ok());

        assert_eq!(container.collected_routes.len(), 0);
    }
}
