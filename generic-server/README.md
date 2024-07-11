# generic-server

This server provides a system for building versatile applications by integrating functionalities offered by configurable plugins.
## Features

* **Plugin System:** Enables developers to extend server behavior through custom plugins.
* **Routing and Transport:** Handles communication between clients and plugins using industry-standard protocols (e.g., HTTP).

# Usage
## Extending Server Functionalities with Plugins
### To add a new plugin:

**1. Implement the `Plugin` Trait:**

 Define the necessary methods (e.g., name(), mount(), and routes()).

**2. Register the Plugin:**

 Add the plugin to the static PLUGINS array (located in `src/plugin/mod.rs`).

**3. Utilize the Plugin:**

 The server will automatically handle routing based on the newly added plugin's routes.

## Example 
### 1- Plugin Implementation
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
### 2- Register the Plugin
```rust
    pub static ref PLUGINS: Vec<Box<dyn Plugin>> = vec![
        #[cfg(feature = "plugin-MyPlugin")]
        Box::<MyPlugin::MyPluginplugin>::default(),
```
