use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub(crate) trait Store: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
}

#[derive(Clone, Default)]
pub(crate) struct InMemoryStore {
    state: Arc<RwLock<HashMap<String, String>>>,
}

impl Store for InMemoryStore {
    fn get(&self, key: &str) -> Option<String> {
        let state = self.state.read().unwrap();
        state.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) {
        let mut state = self.state.write().unwrap();
        state.insert(key.to_string(), value.to_string());
    }
}
