use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait Keyed: Clone {
    type Key: std::hash::Hash + Eq + Clone;
    type KeyRef<'s>: std::hash::Hash + Eq
    where
        Self: 's;

    fn key<'s>(&'s self) -> Self::KeyRef<'s>;
    fn clone_key(r: Self::KeyRef<'_>) -> Self::Key;
}

#[derive(Debug, Clone)]
pub struct Table<V: Keyed> {
    map: HashMap<V::Key, V>,
}

impl<V: Keyed> Table<V> {
    pub fn diff(&self, other: &Table<V>) -> TableDiff<V, V::Key> {
        let mut added = Vec::new();
        let mut changed = Vec::new();
        let mut removed = Vec::new();
        for (key, value) in self.map.iter() {
            if other.map.contains_key(key) {
                changed.push(value.clone());
            } else {
                removed.push(key.clone());
            }
        }
        for (key, value) in other.map.iter() {
            if !self.map.contains_key(key) {
                added.push(value.clone());
            }
        }
        TableDiff {
            added,
            changed,
            removed,
        }
    }

    pub fn apply(&mut self, diff: TableDiff<V, V::Key>) {
        for value in diff.added {
            self.map.insert(V::clone_key(value.key()), value);
        }
        for value in diff.changed {
            self.map.insert(V::clone_key(value.key()), value);
        }
        for key in diff.removed {
            self.map.remove(&key);
        }
    }
}

impl<V: Keyed> From<Vec<V>> for Table<V> {
    fn from(items: Vec<V>) -> Self {
        let mut map = HashMap::new();
        for item in items {
            map.insert(V::clone_key(item.key()), item);
        }
        Self { map }
    }
}

impl<V: Keyed> Default for Table<V> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl<V: Keyed> Serialize for Table<V>
where
    V: Serialize,
    V::Key: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.map.serialize(serializer)
    }
}

impl<'de, V: Keyed> Deserialize<'de> for Table<V>
where
    V: Deserialize<'de>,
    V::Key: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = HashMap::deserialize(deserializer)?;
        Ok(Table { map })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDiff<V, K> {
    pub added: Vec<V>,
    pub changed: Vec<V>,
    pub removed: Vec<K>,
}
