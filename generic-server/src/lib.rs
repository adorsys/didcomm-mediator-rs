pub mod plugin;
pub mod util;

use plugin::container::PluginContainer;

use axum::Router;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;

pub fn app() -> Router {
    let mut container = PluginContainer::default();
    let _ = container.load();

    Router::new() //
        .merge(container.routes().unwrap_or_default())
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
}
