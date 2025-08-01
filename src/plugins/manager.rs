use axum::Router;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use super::PLUGINS;
use plugin_api::Plugin;

#[derive(Debug, PartialEq, Error)]
pub enum PluginContainerError {
    #[error("duplicate entry in plugin registry")]
    DuplicateEntry,
    #[error("plugin container is unloaded")]
    Unloaded,
    #[error("{0}")]
    ContainerError(String),
}

pub struct PluginContainer<'a> {
    loaded: bool,
    collected_routes: Vec<Router>,
    plugins: &'a Vec<Arc<Mutex<dyn Plugin>>>,
    mounted_plugins: Vec<Arc<Mutex<dyn Plugin>>>,
}

impl Default for PluginContainer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginContainer<'_> {
    /// Instantiate an object aware of all statically registered plugins
    pub(crate) fn new() -> Self {
        Self {
            loaded: false,
            collected_routes: vec![],
            plugins: &PLUGINS,
            mounted_plugins: vec![],
        }
    }

    /// Search loaded plugin based on name string
    pub fn find_plugin(&self, name: &str) -> Option<Arc<Mutex<dyn Plugin>>> {
        self.plugins.iter().find_map(|arc_plugin| {
            let plugin = arc_plugin.lock().unwrap();
            (plugin.name() == name).then_some(Arc::clone(arc_plugin))
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

        // Reset collection of routes and mounted plugins
        self.collected_routes.clear();
        self.mounted_plugins.clear();

        // Mount plugins and collect routes on successful status
        let mut errors = HashMap::new();
        self.mounted_plugins.reserve(self.plugins.len());
        self.collected_routes.reserve(self.plugins.len());
        for plugin in self.plugins.iter() {
            let plugin_clone = plugin.clone();
            let mut plugin = plugin.lock().unwrap();
            let plugin_name = plugin.name().to_string();
            match plugin.mount() {
                Ok(_) => {
                    tracing::info!("mounted plugin {}", plugin_name);
                    self.mounted_plugins.push(plugin_clone);
                    self.collected_routes.push(plugin.routes().map_err(|err| {
                        PluginContainerError::ContainerError(format!(
                            "Error collecting routes for plugin {plugin_name}\n{err:?}"
                        ))
                    })?);
                }
                Err(err) => {
                    tracing::error!("Error mounting plugin {plugin_name}\n{:?}", err);
                    errors.insert(plugin_name, err);
                }
            }
        }

        // Flag as loaded
        self.loaded = true;

        // Return state of completion
        if errors.is_empty() {
            tracing::debug!("plugin container loaded");
            Ok(())
        } else {
            Err(PluginContainerError::ContainerError(
                "error loading plugin container".to_string(),
            ))
        }
    }

    /// unload container plugins
    pub fn unload(&mut self) -> Result<(), PluginContainerError> {
        // Check if plugins are loaded before attempting to unload
        if !self.loaded {
            return Err(PluginContainerError::Unloaded);
        }

        // Unmount plugins and clearing the vector of routes
        let errors: HashMap<_, _> = self
            .mounted_plugins
            .iter()
            .filter_map(|plugin| {
                let plugin = plugin.lock().unwrap();
                match plugin.unmount() {
                    Ok(_) => {
                        tracing::info!("unmounted plugin {}", plugin.name());
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

        // Clear mounted plugins and collected routes
        self.mounted_plugins.clear();
        self.collected_routes.clear();

        // Return state of completion
        if errors.is_empty() {
            tracing::debug!("plugin container unloaded");
            Ok(())
        } else {
            Err(PluginContainerError::ContainerError(
                "error unloading plugin container".to_string(),
            ))
        }
    }

    /// Merge collected routes from all plugins successfully initialized.
    pub fn routes(&self) -> Result<Router, PluginContainerError> {
        if self.loaded {
            Ok(self
                .collected_routes
                .iter()
                .fold(Router::new(), |acc: Router, e| acc.merge(e.clone())))
        } else {
            Err(PluginContainerError::Unloaded)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use plugin_api::PluginError;

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

        fn routes(&self) -> Result<Router, PluginError> {
            Ok(Router::new().route("/first", get(|| async {})))
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

        fn routes(&self) -> Result<Router, PluginError> {
            Ok(Router::new().route("/second", get(|| async {})))
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

        fn routes(&self) -> Result<Router, PluginError> {
            Ok(Router::new().route("/second", get(|| async {})))
        }
    }

    struct FaultyPlugin;
    impl Plugin for FaultyPlugin {
        fn name(&self) -> &'static str {
            "faulty"
        }

        fn mount(&mut self) -> Result<(), PluginError> {
            Err(PluginError::InitError("failed to mount".to_owned()))
        }

        fn unmount(&self) -> Result<(), PluginError> {
            Ok(())
        }

        fn routes(&self) -> Result<Router, PluginError> {
            Ok(Router::new().route("/faulty", get(|| async {})))
        }
    }

    #[test]
    fn test_loading() {
        // Mock plugins for testing
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
            mounted_plugins: vec![],
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
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
            mounted_plugins: vec![],
        };

        // Test loading plugins twice
        assert!(container.load().is_ok());
        assert!(container.load().is_ok()); // Load again, should succeed without errors

        // Verify collected routes
        assert_eq!(container.collected_routes.len(), 2);
    }

    #[test]
    fn test_loading_with_duplicates() {
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
            Arc::new(Mutex::new(SecondAgainPlugin {})),
        ];

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
            mounted_plugins: vec![],
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
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(FaultyPlugin {})),
        ];

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
            mounted_plugins: vec![],
        };

        let err = container.load().unwrap_err();

        // Prepare expected error map
        let mut expected_error_map = HashMap::new();
        expected_error_map.insert("faulty".to_string(), PluginError::InitError);

        assert_eq!(
            err,
            PluginContainerError::ContainerError("error loading plugin container".to_string(),)
        );

        // Verify collected routes
        assert_eq!(container.collected_routes.len(), 1);
    }

    #[test]
    fn test_route_extraction_without_loading() {
        // Mock plugins for testing
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];

        // Initialize PluginContainer with the mock plugins
        let container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
            mounted_plugins: vec![],
        };

        // Test route extraction without loading
        assert_eq!(
            container.routes().unwrap_err(),
            PluginContainerError::Unloaded
        );
    }

    #[test]
    fn test_unloading() {
        // Mock plugins for testing
        let plugins: Vec<Arc<Mutex<dyn Plugin>>> = vec![
            Arc::new(Mutex::new(FirstPlugin {})),
            Arc::new(Mutex::new(SecondPlugin {})),
        ];
        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
            mounted_plugins: vec![],
        };
        // Test unloading plugins
        assert!(container.load().is_ok());

        // Verify collected routes
        assert_eq!(container.collected_routes.len(), 2);

        // unloading container and clearing routes
        assert!(container.unload().is_ok());
        assert_eq!(container.collected_routes.len(), 0);
    }
}
