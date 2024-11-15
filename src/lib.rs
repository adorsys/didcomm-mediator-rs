pub mod plugins;

use axum::Router;
use plugins::manager::PluginContainer;
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
