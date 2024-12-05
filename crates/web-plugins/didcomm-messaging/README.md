# didcomm-messaging

The `didcomm-messaging` plugin is a web plugin of the [DIDComm mediator](https://github.com/adorsys/didcomm-mediator-rs/) project. It provides implementations of various DIDComm messaging protocols as plugins and features, so protocols can be added and deleted with minimal effort as well as being dynamically loaded.

See the repository [README](https://github.com/adorsys/didcomm-mediator-rs/blob/main/README.md) for the list of currently supported protocols.

## Usage

**Implementing a new protocol:**

* Define handler(s) for the protocol

```rust
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use didcomm::Message;
use message_api::{MessageHandler, MessagePlugin, MessageRouter};
use shared::state::AppState;
use std::sync::Arc;

pub struct MockProtocol;

struct ExampleProtocolHandler1;
struct ExampleProtocolHandler2;

#[async_trait]
impl MessageHandler for ExampleProtocolHandler1 {
    async fn handle(
        &self,
        _state: Arc<AppState>,
        message: Message,
    ) -> Result<Option<Message>, Response> {
        // do something with the message
        Ok(None)
    }
}

#[async_trait]
impl MessageHandler for ExampleProtocolHandler2 {
    async fn handle(
        &self,
        _state: Arc<AppState>,
        message: Message,
    ) -> Result<Option<Message>, Response> {
        // do something with the message
        Ok(None)
    }
}

impl MessagePlugin for MockProtocol {
    fn name(&self) -> &'static str {
        "mock_protocol"
    }

    fn didcomm_routes(&self) -> MessageRouter {
        MessageRouter::new()
            .register("message-type-1", ExampleProtocolHandler1)
            .register("message-type-2", ExampleProtocolHandler2)
    }
}
```

* Add the protocol to the `DIDCOMM_PLUGINS` array in `crates/web-plugins/didcomm-messaging/src/protocols.rs`:

```rust
pub(crate) static DIDCOMM_PLUGINS: Lazy<Vec<Arc<dyn MessagePlugin>>> = Lazy::new(|| {
    vec![
        #[cfg(feature = "mock-protocol")]
        Arc::new(MockProtocol),
        // other plugins
    ]
});
```

* Add the plugin as a feature in `didcomm-messaging` `Cargo.toml`:

```toml
[dependencies]
mock-protocol = { workspace = true, optional = true }

[features]
mock-protocol = ["mock-protocol", ...]

default = ["mock-protocol", ...]
```

* Adjust the workspace `Cargo.toml`:

```toml
[workspace.dependencies]
mock-protocol = { path = "./crates/web-plugins/didcomm-messaging/protocols/mock-protocol", version = "0.1.0" }

[features]
mock-protocol = ["plugin-didcomm_messaging", "didcomm-messaging/mock-protocol"]
```

The plugin manager will automatically handle routing based on the added protocol's routes.
