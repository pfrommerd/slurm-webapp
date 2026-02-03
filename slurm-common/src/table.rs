use crate::{Job, Node, Partition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub trait HasId {
    fn id(&self) -> Uuid;
}

impl HasId for Node {
    fn id(&self) -> Uuid {
        self.id
    }
}
impl HasId for Partition {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl HasId for Job {
    fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table<V: HasId> {
    map: HashMap<Uuid, V>,
}

impl<V: HasId + Clone> Table<V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, value: V) {
        self.map.insert(value.id(), value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &V)> {
        self.map.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.map.values()
    }

    pub fn diff(&self, other: &Table<V>) -> TableDiff<V> {
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

    pub fn apply(&mut self, diff: TableDiff<V>) {
        for value in diff.added {
            self.map.insert(value.id(), value);
        }
        for value in diff.changed {
            self.map.insert(value.id(), value);
        }
        for key in diff.removed {
            self.map.remove(&key);
        }
    }
}

impl<V: HasId> From<Vec<V>> for Table<V> {
    fn from(items: Vec<V>) -> Self {
        let mut map = HashMap::new();
        for item in items {
            map.insert(item.id(), item);
        }
        Self { map }
    }
}

impl<V: HasId> Default for Table<V> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl<V: HasId> Serialize for Table<V>
where
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let values: Vec<&V> = self.map.values().collect();
        values.serialize(serializer)
    }
}

impl<'de, V: HasId> Deserialize<'de> for Table<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let values: Vec<V> = Vec::deserialize(deserializer)?;
        Ok(Table::from(values))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableDiff<V> {
    pub added: Vec<V>,
    pub changed: Vec<V>,
    pub removed: Vec<Uuid>,
}
