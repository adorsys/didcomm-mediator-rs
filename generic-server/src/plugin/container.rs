use std::sync::Mutex;
use std::collections::{HashMap, HashSet};

use axum::Router;
use server_plugin::{Plugin, PluginError};

use super::PLUGINS;

#[derive(Debug, PartialEq)]
pub enum PluginContainerError {
    DuplicateEntry,
    Unloaded,
    PluginErrorMap(HashMap<String, PluginError>),
}

pub struct PluginContainer<'a> {
    loaded: bool,
    collected_routes: Vec<Router>,
    plugins: &'a Mutex<Vec<Box<dyn Plugin + Send>>>,
}

impl<'a> Default for PluginContainer<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PluginContainer<'a> {
    /// Instantiate an object aware of all statically registered plugins
    pub fn new() -> Self {
        Self {
            loaded: false,
            collected_routes: vec![],
            plugins: &*PLUGINS,
        }
    }

    /// Search loaded plugin based on name string
    pub fn find_plugin(&self, name: &str) -> Option<usize> {
        let plugins = self.plugins.lock().unwrap();
        plugins.iter().position(|plugin| name == plugin.name())
    }    
    
    /// Load referenced plugins
    ///
    /// This entails mounting them and merging their routes internally (only
    /// upon successful initialization). An error is returned if plugins
    /// bearing the same name are found. Also, all plugins failing to be
    /// initialized are returned in a map with respectively raised errors.
    pub fn load(&mut self) -> Result<(), PluginContainerError> {
        tracing::debug!("loading plugin container");

        // Obtain lock
        let mut plugins = self.plugins.lock().unwrap();

        // Checking for duplicates
        let mut seen_names = HashSet::new();
        for plugin in plugins.iter() {
            if !seen_names.insert(plugin.name().to_string()) {
                tracing::error!("found duplicate entry in plugin registry: {}", plugin.name());
                return Err(PluginContainerError::DuplicateEntry);
            }
        }

        // Reset collection of routes
        self.collected_routes.clear();

        // Mount plugins and collect routes on successful status
        let errors: HashMap<_, _> = plugins
            .iter_mut()
            .filter_map(|plugin| match plugin.mount() {
                Ok(_) => {
                    tracing::info!("mounted plugin {}", plugin.name());
                    self.collected_routes.push(plugin.routes());
                    None
                }
                Err(err) => {
                    tracing::error!("error mounting plugin {}", plugin.name());
                    Some((plugin.name().to_string(), err))
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
        let plugins: Mutex<Vec<Box<dyn Plugin + Send>>> = Mutex::new(vec![
            Box::new(FirstPlugin {}),
            Box::new(SecondPlugin {}),
        ]);

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
        };

        // Test loading plugins
        assert!(container.load().is_ok());
        assert!(container.routes().is_ok());

        // Verify find_plugin method
        assert!(container.find_plugin("first").is_some());
        assert!(container.find_plugin("second").is_some());
        assert!(container.find_plugin("non-existent").is_none());

        // Verify collected routes
        assert_eq!(container.collected_routes.len(), 2);
    }

    #[test]
    fn test_double_loading() {
        // Mock plugins for testing
        let plugins: Mutex<Vec<Box<dyn Plugin + Send>>> = Mutex::new(vec![
            Box::new(FirstPlugin {}),
            Box::new(SecondPlugin {}),
        ]);

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
        };

        // Test loading plugins twice
        assert!(container.load().is_ok());
        assert!(container.load().is_ok());

        // Verify collected routes
        assert_eq!(container.collected_routes.len(), 2);
    }

    #[test]
    fn test_loading_with_duplicates() {
        // Mock plugins for testing
        let plugins: Mutex<Vec<Box<dyn Plugin + Send>>> = Mutex::new(vec![
            Box::new(SecondPlugin {}),
            Box::new(SecondAgainPlugin {}),
        ]);

        // Initialize PluginContainer with the mock plugins
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
        };

        // Test loading plugins with duplicate names
        assert_eq!(
            container.load().unwrap_err(),
            PluginContainerError::DuplicateEntry
        );
    }

    #[test]
    fn test_loading_with_failing_plugin() {
           // Mock plugins for testing
           let plugins: Mutex<Vec<Box<dyn Plugin + Send>>> = Mutex::new(vec![
            Box::new(FirstPlugin {}),
            Box::new(FaultyPlugin {}),
        ]);
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
        };

        let err = container.load().unwrap_err();

        assert_eq!(
            err,
            PluginContainerError::PluginErrorMap(
                [("faulty".to_string(), PluginError::InitError)]
                    .into_iter()
                    .collect()
            )
        );

        assert_eq!(container.collected_routes.len(), 1);
    }

    #[test]
    fn test_route_extraction_without_loading() {
        // Mock plugins for testing
        let plugins: Mutex<Vec<Box<dyn Plugin + Send>>> = Mutex::new(vec![
            Box::new(FirstPlugin {}),
            Box::new(SecondPlugin {}),
        ]);

        // Initialize PluginContainer with the mock plugins
        let container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &plugins,
        };

        // Test route extraction without loading
        assert_eq!(
            container.routes().unwrap_err(),
            PluginContainerError::Unloaded
        );
    }
}
