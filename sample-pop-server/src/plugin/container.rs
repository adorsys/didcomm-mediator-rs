use std::collections::{HashMap, HashSet};

use axum::Router;

use super::{
    traits::{Plugin, PluginError},
    PLUGINS,
};

#[derive(Debug, PartialEq)]
pub enum PluginContainerError {
    DuplicateEntry,
    Unloaded,
    PluginErrorMap(HashMap<String, PluginError>),
}

pub struct PluginContainer<'a> {
    loaded: bool,
    collected_routes: Vec<Router>,
    plugins: &'a Vec<Box<dyn Plugin>>,
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
    pub fn find_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find_map(|plugin| (name == plugin.name()).then_some(&**plugin))
    }

    /// Load referenced plugins
    ///
    /// This entails mounting them and merging their routes internally (only
    /// upon successful initialization). An error is returned if plugins
    /// bearing the same name are found. Also, all plugins failing to be
    /// initialized are returned in a map with respectively raised errors.
    pub fn load(&mut self) -> Result<(), PluginContainerError> {
        tracing::debug!("loading plugin container");

        // Checking for duplicates
        let unique_plugins: HashSet<_> = self.plugins.iter().collect();
        if unique_plugins.len() != self.plugins.len() {
            tracing::error!("found duplicate entries in plugin registry");
            return Err(PluginContainerError::DuplicateEntry);
        }

        // Reset collection of routes
        self.collected_routes.truncate(0);

        // Mount plugins and collect routes on successful status
        let errors: HashMap<_, _> = self
            .plugins
            .iter()
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

    struct FirstPlugin;
    impl Plugin for FirstPlugin {
        fn name(&self) -> &'static str {
            "first"
        }

        fn mount(&self) -> Result<(), PluginError> {
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

        fn mount(&self) -> Result<(), PluginError> {
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

        fn mount(&self) -> Result<(), PluginError> {
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

        fn mount(&self) -> Result<(), PluginError> {
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
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &vec![Box::new(FirstPlugin {}), Box::new(SecondPlugin {})],
        };

        assert!(container.load().is_ok());
        assert!(container.routes().is_ok());

        assert!(container.find_plugin("first").is_some());
        assert!(container.find_plugin("second").is_some());
        assert!(container.find_plugin("non-existent").is_none());

        // The actual routes collected are actually hard to test
        // given that axum::Router seems not to provide public
        // directives to inquire internal state.
        // See: https://github.com/tokio-rs/axum/discussions/860
        assert_eq!(container.collected_routes.len(), 2);
    }

    #[test]
    fn test_double_loading() {
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &vec![Box::new(FirstPlugin {}), Box::new(SecondPlugin {})],
        };

        assert!(container.load().is_ok());
        assert!(container.load().is_ok());

        assert_eq!(container.collected_routes.len(), 2);
    }

    #[test]
    fn test_loading_with_duplicates() {
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &vec![Box::new(SecondPlugin {}), Box::new(SecondAgainPlugin {})],
        };

        assert_eq!(
            container.load().unwrap_err(),
            PluginContainerError::DuplicateEntry
        );
    }

    #[test]
    fn test_loading_with_failing_plugin() {
        let mut container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &vec![Box::new(FirstPlugin {}), Box::new(FaultyPlugin {})],
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
        let container = PluginContainer {
            loaded: false,
            collected_routes: vec![],
            plugins: &vec![Box::new(FirstPlugin {}), Box::new(SecondPlugin {})],
        };

        assert_eq!(
            container.routes().unwrap_err(),
            PluginContainerError::Unloaded
        );
    }
}
