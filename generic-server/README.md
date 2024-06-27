# generic-server

This server provides a framework to build versatile applications by aggregating functionalities offered by configurable plugins.

## Features

* **Plugin System:** Enables developers to extend server behavior through custom plugins.
* **Routing and Transport:** Handles communication between clients and plugins using industry-standard protocols (e.g., HTTP).

## Usage

```rust
     name(&self) -> &'static str {
        "greeter"
    }

    // Called when the plugin is mounted to the server
     mount(&self) -> Result<(), PluginError> {
        // Perform any setup for the plugin (e.g., database connection)
        Ok(())
    }

    // Called when the plugin is unmounted from the server
     unmount(&self) -> Result<(), PluginError> {
        // Perform any cleanup needed for the plugin (e.g., closing database connection)
        Ok(())
    }

    // Defines the routes for the plugin
     routes(&self) -> Router {
        // Define routes using web::routes function
        web::routes()
        }
    
