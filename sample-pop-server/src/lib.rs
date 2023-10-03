pub mod didgen;
pub mod plugin;
pub mod util;
pub mod web;

use plugin::container::PluginContainer;

use axum::Router;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;

#[allow(unused)]
pub const DIDDOC_DIR: &str = "storage";
pub const KEYSTORE_DIR: &str = "storage/keystore";

pub fn app() -> Router {
    let mut loader = PluginContainer::default();
    let _ = loader.load();

    web::routes() //
        .merge(loader.routes().unwrap_or_default())
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
}
