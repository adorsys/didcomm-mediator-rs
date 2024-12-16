pub mod plugins;

use axum::Router;
use eyre::{eyre, Result};
use hyper::Method;
use plugins::manager::PluginContainer;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn app() -> Result<(PluginContainer<'static>, Router)> {
    let mut container = PluginContainer::new();
    container.load().map_err(|e| eyre!(e))?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let router = Router::new()
        .merge(container.routes().unwrap_or_default())
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
        .layer(cors);

    Ok((container, router))
}