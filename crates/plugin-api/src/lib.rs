use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};
use thiserror::Error;

use axum::Router;

#[derive(Debug, Error, PartialEq)]
pub enum PluginError {
    #[error("{0}")]
    InitError(String),
    #[error("{0}")]
    Other(String),
}

pub trait Plugin: Sync + Send {
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Provide initialization actions as needed
    fn mount(&mut self) -> Result<(), PluginError>;

    /// Revert initialization actions as needed
    fn unmount(&self) -> Result<(), PluginError>;

    /// Export managed endpoints
    fn routes(&self) -> Result<Router, PluginError>;
}

impl Eq for dyn Plugin {}

impl PartialEq for dyn Plugin {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Hash for dyn Plugin {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name().hash(state)
    }
}
