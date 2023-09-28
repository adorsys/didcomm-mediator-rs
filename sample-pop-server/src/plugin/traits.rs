use std::hash::{Hash, Hasher};

use axum::Router;

pub enum PluginError {
    InitError,
    Unloaded,
}

pub trait Plugin: Sync {
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Provide initialization actions as needed
    fn initialize(&self) -> Result<(), PluginError>;

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
