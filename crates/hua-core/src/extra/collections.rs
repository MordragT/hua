use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

#[derive(Debug, Default)]
pub struct OrdValTreeMap<K, V: Ord> {
    pub tree: BTreeMap<V, K>,
    pub indices: HashMap<K, V>,
}

impl<K: Copy + Eq + Hash, V: Ord + Clone> OrdValTreeMap<K, V> {
    pub fn new() -> Self {
        Self {
            tree: BTreeMap::new(),
            indices: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let val = value.clone();
        self.tree.insert(value, key);
        self.indices.insert(key, val)
    }
}
