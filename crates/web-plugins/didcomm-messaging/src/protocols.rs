use std::{
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum PluginError {
    InitError,
}

#[derive(Debug)]
pub struct ProtocolRouter<F, T, M, R>
where
    M: Serialize + for<'de> Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
    pub route_map: HashMap<String, F>,
    _marker: PhantomData<(T, M, R)>,
}

impl<F, T, M, R> ProtocolRouter<F, T, M, R>
where
    M: Serialize + for<'de> Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            route_map: HashMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn add_route(mut self, msg_type: &str, handler: F) -> Self {
        self.route_map.insert(msg_type.to_string(), handler);
        self
    }

    pub fn merge(self, other: ProtocolRouter<F, T, M, R>) -> Self {
        let mut new = self;
        for (k, v) in other.route_map.into_iter() {
            new.route_map.insert(k, v);
        }
        new
    }
}

// Implement Default for ProtocolRouter
impl<F, T, M, R> Default for ProtocolRouter<F, T, M, R>
where
    M: Serialize + for<'de> Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait Plugin<F, T, M, R>: Sync + Send
where
    M: Serialize + for<'de> Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
    /// Define a unique identifier
    fn name(&self) -> &'static str;

    /// Provide initialization actions as needed
    fn mount(&mut self) -> Result<(), PluginError>;

    /// Revert initialization actions as needed
    fn unmount(&self) -> Result<(), PluginError>;

    /// Return a mapping of message types to handlers
    fn get_routes(&self) -> ProtocolRouter<F, T, M, R>;
}

impl<F, T, M, R> Eq for dyn Plugin<F, T, M, R>
where
    M: Serialize + for<'de> Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
}

impl<F, T, M, R> PartialEq for dyn Plugin<F, T, M, R>
where
    M: Serialize + for<'de> Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl<F, T, M, R> Hash for dyn Plugin<F, T, M, R>
where
    M: Serialize + for<'de> Deserialize<'de>,
    F: Fn(&T, &M) -> Result<M, R> + Send + Sync + 'static,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name().hash(state)
    }
}
