pub mod plugins;

use axum::Router;
use eyre::{eyre, Result};
use plugins::handler::PluginContainer;
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};

pub fn app() -> Result<(PluginContainer<'static>, Router)> {
    let mut container = PluginContainer::new();
    container.load().map_err(|e| eyre!(e))?;

    let router = Router::new()
        .merge(container.routes().unwrap_or_default())
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new());

    Ok((container, router))
}
