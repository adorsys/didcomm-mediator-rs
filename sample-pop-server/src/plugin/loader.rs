use std::{collections::HashMap, vec};

use axum::Router;

use super::{
    traits::{Plugin, PluginError},
    PLUGINS,
};

pub struct PluginLoader {
    loaded: bool,
    plugins: &'static Vec<Box<dyn Plugin>>,
    collected_routes: Vec<Router>,
}

impl PluginLoader {
    /// Instantiate an object aware of all statically registered plugins
    pub fn new() -> Self {
        Self {
            loaded: false,
            plugins: &*PLUGINS,
            collected_routes: vec![],
        }
    }

    /// Load referenced plugins
    ///
    /// This entails initializing them and merging their routes internally (only
    /// upon successful initialization). All plugins failing to be initialized
    /// are returned in a map with respectively raised errors.
    pub fn load(&mut self) -> Result<(), HashMap<&dyn Plugin, PluginError>> {
        // Reset collection of routes
        self.collected_routes.truncate(0);

        // Initialize plugins and collect routes on successful status
        let errors: HashMap<_, _> = self
            .plugins
            .iter()
            .filter_map(|plugin| match plugin.initialize() {
                Ok(_) => {
                    self.collected_routes.push(plugin.routes());
                    None
                }
                Err(err) => Some((&**plugin, err)),
            })
            .collect();

        // Flag as loaded
        self.loaded = true;

        // Return state of completion
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Merge collected routes from all plugins successfully initialized.
    pub fn routes(&self) -> Result<Router, PluginError> {
        if self.loaded {
            Ok(self
                .collected_routes
                .iter()
                .fold(Router::new(), |acc, e| acc.merge(e.clone())))
        } else {
            Err(PluginError::Unloaded)
        }
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
