# Server plugin API

The server used for this project provides a system for building versatile applications by integrating functionalities offered by configurable plugins via the plugin API.

## Features

* **Plugin System:** The server's behavior can be extended through custom plugins.
* **Routing and Transport:** Handles communication between clients and plugins.

## Usage

### Adding a new plugin

You can add a new plugin by following the steps below:

**1. Implement the `Plugin` trait**

First, import the [`Plugin`](../plugin-api/src/lib.rs) trait located inside the [plugin-api](../plugin-api) crate. Then,
define the necessary methods such as `name()`, `mount()`, `unmount()` and `routes()`.

**2. Register the Plugin**  

Add the plugin to the static **`PLUGINS`** array located in `src/plugins.rs`.  

The server will automatically handle routing based on the added plugin's routes.

### Example

* **Plugin Implementation**

Here's an example of how to implement a plugin:

```rust
use axum::{routing::get, Router};
use server_plugin::{Plugin, PluginError};

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
        // Deinitialization logic here
        Ok(())
    }

    fn routes(&self) -> Router {
        // Define and return routes here
        Router::new().route("/myplugin", get(my_plugin_handler))
    }
}

async fn my_plugin_handler() -> &'static str {
    "Hello from MyPlugin!"
}
```

* **Register the Plugin**

To register the plugin, add it to the `PLUGINS` array:

```rust
lazy_static! {
    pub(crate) static ref PLUGINS: Vec<Arc<Mutex<dyn Plugin + 'static>>> = vec![
        #[cfg(feature = "plugin-myplugin")]
        Arc::new(Mutex::new(example::plugin::MyPlugin {})),
    ];
}
```
