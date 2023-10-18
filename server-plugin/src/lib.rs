use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use axum::Router;

#[derive(Debug, PartialEq)]
pub enum PluginError {
    InitError,
}

pub trait Plugin: Sync {
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Provide initialization actions as needed
    fn mount(&self) -> Result<(), PluginError>;

    /// Revert initialization actions as needed
    fn unmount(&self) -> Result<(), PluginError>;

    /// Export managed endpoints
    fn routes(&self) -> Router;
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
