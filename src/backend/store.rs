use dashmap::DashMap;
use dashmap::DashSet;

use crate::RespFrame;

#[derive(Debug)]
pub struct Store {
    pub(crate) hmap: DashMap<String, DashMap<String, RespFrame>>,
    pub(crate) hset: DashMap<String, DashSet<RespFrame>>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            hmap: DashMap::<String, DashMap<String, RespFrame>>::new(),
            hset: DashMap::<String, DashSet<RespFrame>>::new(),
        }
    }
}

impl Store {
    pub fn new() -> Self {
        Store::default()
    }
}
