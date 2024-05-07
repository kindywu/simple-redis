mod store;

use std::{ops::Deref, sync::Arc};

pub use store::*;

#[derive(Debug, Clone)]
pub struct Backend(Arc<Store>);

impl Deref for Backend {
    type Target = Store;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(Store::default()))
    }
}

impl Backend {
    pub fn new() -> Self {
        Backend::default()
    }
}
