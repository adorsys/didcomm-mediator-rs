mod util;
mod web;

use axum::Router;
use plugin_api::{Plugin, PluginError};

#[derive(Default)]
pub(crate) struct IndexPlugin;

impl Plugin for IndexPlugin {
    fn name(&self) -> &'static str {
        "index"
    }

    fn mount(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Result<Router, PluginError> {
        Ok(web::routes())
    }
}
