mod web;

use axum::Router;
use server_plugin::{Plugin, PluginError};

#[derive(Default)]
pub struct IndexPlugin;

impl Plugin for IndexPlugin {
    fn name(&self) -> &'static str {
        "index"
    }

    fn mount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        web::routes()
    }
}
