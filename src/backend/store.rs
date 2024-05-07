use dashmap::DashMap;
use dashmap::DashSet;

use crate::RespFrame;

#[derive(Debug)]
pub struct Store {
    pub(crate) hset: DashMap<String, DashSet<RespFrame>>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            hset: DashMap::<String, DashSet<RespFrame>>::new(),
        }
    }
}

impl Store {
    pub fn new() -> Self {
        Store::default()
    }
}
