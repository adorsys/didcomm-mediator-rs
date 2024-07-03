pub mod plugin;
pub mod util;
use once_cell::unsync::Lazy;
use plugin::container::PluginContainer;

use axum::Router;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;
// creating plugincontainer globally for easy loading and unloading
static mut CONTAINER: Lazy<Option<PluginContainer>> = Lazy::new(|| None);

pub fn app() -> Router {
    let mut container = PluginContainer::default();
    let _ = container.load();
    unsafe { CONTAINER.replace(container) };
    unsafe {
        Router::new() //
            .merge(CONTAINER.as_ref().unwrap().routes().unwrap_or_default())
            .layer(TraceLayer::new_for_http())
            .layer(CatchPanicLayer::new())
    }
}

// creating function to unmount plugins on shutdown
pub fn unload_for_shutdown() {
    unsafe {
        CONTAINER.as_mut().unwrap().unload().unwrap();
    }
    
}


