# generic-server

This server provides a system for building versatile applications by integrating functionalities offered by configurable plugins.
## Features

* **Plugin System:** Enables developers to extend server behavior through custom plugins.
* **Routing and Transport:** Handles communication between clients and plugins using industry-standard protocols (e.g., HTTP).

## Usage

```rust
      // initialisation: 
      //This initializes the container with default values and references the registered plugins
        Self {
            loaded: false,
            collected_routes: vec![],
            plugins: &*PLUGINS,
        }

    // Finding Plugins:
    //This method searches for a plugin by its name.
        find_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find_map(|plugin| (name == plugin.name()).then_some(&**plugin))
    }

    //Loading Plugins
    //This method loads all referenced plugins, checks for duplicates, mounts them, collects routes, and handles errors.
    pub fn load(&mut self) -> Result<(), PluginContainerError> {
    // ... (code truncated for brevity)
}

    // Merging Routes:
   //This method merges routes from all successfully initialized plugins into a single Router.


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
```
# Extending Server Functionalities with Plugins
## To add a new plugin:

**1- Implement the `Plugin` Trait:**

- Define the necessary methods (e.g., name(), mount(), and routes()).

**2-Register the Plugin:**

- Add the plugin to the static PLUGINS array.

**3-Load the Plugin:**

- Ensure the plugin container is re-initialized to include the new plugin.**

**4- Utilize the Plugin:**

- The server will automatically handle routing based on the newly added plugin's routes.

## Example Plugin Implementation
```rust
struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "MyPlugin"
    }

    fn mount(&self) -> Result<(), PluginError> {
        // Initialization logic here
        Ok(())
    }
    fn unmount(&self) -> Result<(), PluginError> {
        // Initialization logic here
        Ok(())
    }
    fn routes(&self) -> Router {
        // Define and return routes here
        Router::new().route("/myplugin", get(my_plugin_handler))
    }
}
```
