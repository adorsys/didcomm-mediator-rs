pub mod model;
pub mod util;
pub mod web;

use axum::Router;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;

#[allow(unused)]
pub const DIDDOC_DIR: &str = "storage";
pub const KEYSTORE_DIR: &str = "storage/keystore";

pub fn app() -> Router {
    web::routes() //
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
}
