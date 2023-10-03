pub mod didgen;
mod util;
mod web;

use super::traits::{Plugin, PluginError};
use axum::Router;

pub const DIDDOC_DIR: &str = "storage";
pub const KEYSTORE_DIR: &str = "storage/keystore";

#[derive(Default)]
pub struct DidPopPlugin;

impl Plugin for DidPopPlugin {
    fn name(&self) -> &'static str {
        "didpop"
    }

    fn mount(&self) -> Result<(), PluginError> {
        if didgen::validate_diddoc().is_err() {
            didgen::didgen().map_err(|_| {
                tracing::error!("failed to generate an initial keystore and its DID document");

                PluginError::InitError
            })?;
        };

        Ok(())
    }

    fn unmount(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn routes(&self) -> Router {
        web::routes()
    }
}
