use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    pub res_id: String, // e.g. "cpu", "gpu"
    pub total: u64,
    pub allocated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub name: String,
    pub state: String, // e.g., "idle", "alloc", "down"
    pub cpus: u32,
    pub real_memory: i64, // in MB, use i64 to be sqlite compatible
    pub resources: HashMap<String, Resource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobState {
    PENDING,
    RUNNING,
    COMPLETED,
    FAILED,
    CANCELLED,
    UNKNOWN,
}

impl AsRef<str> for JobState {
    fn as_ref(&self) -> &str {
        match self {
            JobState::PENDING => "PENDING",
            JobState::RUNNING => "RUNNING",
            JobState::COMPLETED => "COMPLETED",
            JobState::FAILED => "FAILED",
            JobState::CANCELLED => "CANCELLED",
            JobState::UNKNOWN => "UNKNOWN",
        }
    }
}
impl ToString for JobState {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub job_id: String,
    pub user: String,
    pub partition: String,
    pub state: JobState,
    pub num_nodes: u32,
    pub num_cpus: u32,
    pub time_limit: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub submit_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Partition {
    pub name: String,
    pub total_nodes: u32,
    pub total_cpus: u32,
    pub state: String, // "UP", "DOWN"
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub nodes: Vec<Node>,
    pub jobs: Vec<Job>,
    pub partitions: Vec<Partition>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterDiff {
    pub nodes_upserted: Vec<Node>,
    pub nodes_removed: Vec<String>, // names
    pub jobs_upserted: Vec<Job>,
    pub jobs_removed: Vec<String>, // job_ids
    pub partitions_upserted: Vec<Partition>,
    pub partitions_removed: Vec<String>, // names
    pub updated_at: DateTime<Utc>,
}

impl ClusterStatus {
    pub fn diff(&self, other: &ClusterStatus) -> ClusterDiff {
        ClusterDiff {
            nodes_upserted: diff_upsert(&self.nodes, &other.nodes, |n| &n.name),
            nodes_removed: diff_remove(&self.nodes, &other.nodes, |n| &n.name),
            jobs_upserted: diff_upsert(&self.jobs, &other.jobs, |j| &j.job_id),
            jobs_removed: diff_remove(&self.jobs, &other.jobs, |j| &j.job_id),
            partitions_upserted: diff_upsert(&self.partitions, &other.partitions, |p| &p.name),
            partitions_removed: diff_remove(&self.partitions, &other.partitions, |p| &p.name),
            updated_at: other.updated_at,
        }
    }

    pub fn apply(&mut self, diff: ClusterDiff) {
        // Apply Nodes
        apply_diff(
            &mut self.nodes,
            diff.nodes_upserted,
            diff.nodes_removed,
            |n| n.name.clone(),
        );
        // Apply Jobs
        apply_diff(&mut self.jobs, diff.jobs_upserted, diff.jobs_removed, |j| {
            j.job_id.clone()
        });
        // Apply Partitions
        apply_diff(
            &mut self.partitions,
            diff.partitions_upserted,
            diff.partitions_removed,
            |p| p.name.clone(),
        );

        self.updated_at = diff.updated_at;
    }
}

// Helper to find items in `new` that are different or not present in `old`.
fn diff_upsert<T, F, K>(old: &[T], new: &[T], key_fn: F) -> Vec<T>
where
    T: PartialEq + Clone,
    F: Fn(&T) -> &K,
    K: std::cmp::Eq + std::hash::Hash,
{
    let mut old_map = HashMap::new();
    for item in old {
        old_map.insert(key_fn(item), item);
    }

    let mut upserted = Vec::new();
    for item in new {
        let key = key_fn(item);
        if let Some(old_item) = old_map.get(key) {
            if *old_item != item {
                upserted.push(item.clone());
            }
        } else {
            upserted.push(item.clone());
        }
    }
    upserted
}

// Helper to find items in `old` that are not present in `new`.
fn diff_remove<T, F, K>(old: &[T], new: &[T], key_fn: F) -> Vec<String>
where
    F: Fn(&T) -> &K,
    K: std::cmp::Eq + std::hash::Hash + ToString,
{
    let new_keys: HashSet<_> = new.iter().map(|item| key_fn(item)).collect();
    old.iter()
        .filter_map(|item| {
            let key = key_fn(item);
            if !new_keys.contains(key) {
                Some(key.to_string())
            } else {
                None
            }
        })
        .collect()
}

// Helper to apply diffs to a list
fn apply_diff<T, F>(list: &mut Vec<T>, upserted: Vec<T>, removed: Vec<String>, key_fn: F)
where
    F: Fn(&T) -> String,
{
    // Remove items
    let removed_set: HashSet<_> = removed.into_iter().collect();
    list.retain(|item| !removed_set.contains(&key_fn(item)));

    // Upsert items (replace if exists, add if new)
    for item in upserted {
        let key = key_fn(&item);
        if let Some(pos) = list.iter().position(|x| key_fn(x) == key) {
            list[pos] = item;
        } else {
            list.push(item);
        }
    }
}
