pub mod plugins;
pub mod secret_key;
pub mod secure_key;

use axum::Router;
use plugins::handler::PluginContainer;
use secret_key::SecretBox;
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};

pub fn app() -> (PluginContainer<'static>, Router) {
    let mut container = PluginContainer::new();
    let _ = container.load();

    let router = Router::new()
        .merge(container.routes().unwrap_or_default())
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new());

    (container, router)
}
