use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

#[cfg(feature = "db")]
pub mod db;
pub mod parser;
pub mod scontrol;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    pub res_id: String, // e.g. "cpu", "gres:h200"
    pub total: u64,
    pub allocated: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Idle,
    Alloc,
    Mix,
    Down,
    Unknown,
}

impl FromStr for NodeStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IDLE" => Ok(NodeStatus::Idle),
            "ALLOC" => Ok(NodeStatus::Alloc),
            "MIX" => Ok(NodeStatus::Mix),
            "DOWN" => Ok(NodeStatus::Down),
            "UNKNOWN" => Ok(NodeStatus::Unknown),
            _ => Err(anyhow::anyhow!("Invalid node status: {}", s)),
        }
    }
}
impl AsRef<str> for NodeStatus {
    fn as_ref(&self) -> &str {
        match self {
            NodeStatus::Idle => "IDLE",
            NodeStatus::Alloc => "ALLOC",
            NodeStatus::Mix => "MIX",
            NodeStatus::Down => "DOWN",
            NodeStatus::Unknown => "UNKNOWN",
        }
    }
}
impl ToString for NodeStatus {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub name: String,
    pub status: NodeStatus, // e.g., "idle", "alloc", "down"
    pub cpus: u32,
    pub real_memory: i64, // in MB, use i64 to be sqlite compatible
    pub resources: HashMap<String, Resource>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Unknown,
}

impl FromStr for JobStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PENDING" => Ok(JobStatus::Pending),
            "RUNNING" => Ok(JobStatus::Running),
            "COMPLETED" => Ok(JobStatus::Completed),
            "FAILED" => Ok(JobStatus::Failed),
            "CANCELLED" => Ok(JobStatus::Cancelled),
            "UNKNOWN" => Ok(JobStatus::Unknown),
            _ => Err(anyhow::anyhow!("Invalid job status: {}", s)),
        }
    }
}
impl AsRef<str> for JobStatus {
    fn as_ref(&self) -> &str {
        match self {
            JobStatus::Pending => "PENDING",
            JobStatus::Running => "RUNNING",
            JobStatus::Completed => "COMPLETED",
            JobStatus::Failed => "FAILED",
            JobStatus::Cancelled => "CANCELLED",
            JobStatus::Unknown => "UNKNOWN",
        }
    }
}
impl ToString for JobStatus {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub job_id: String,
    pub user: String,
    pub partition: String,
    pub status: JobStatus,
    pub num_nodes: u32,
    pub num_cpus: u32,
    pub time_limit: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub submit_time: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PartitionStatus {
    Up,
    Down,
    Unknown,
}

impl AsRef<str> for PartitionStatus {
    fn as_ref(&self) -> &str {
        match self {
            PartitionStatus::Up => "UP",
            PartitionStatus::Down => "DOWN",
            PartitionStatus::Unknown => "UNKNOWN",
        }
    }
}
impl FromStr for PartitionStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UP" => Ok(PartitionStatus::Up),
            "DOWN" => Ok(PartitionStatus::Down),
            "UNKNOWN" => Ok(PartitionStatus::Unknown),
            _ => Err(anyhow::anyhow!("Invalid partition state: {}", s)),
        }
    }
}
impl ToString for PartitionStatus {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Partition {
    pub name: String,
    pub total_nodes: u32,
    pub total_cpus: u32,
    pub status: PartitionStatus,
    pub updated_at: DateTime<Utc>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub nodes: Vec<Node>,
    pub jobs: Vec<Job>,
    pub partitions: Vec<Partition>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterDiff {
    pub nodes_upserted: Vec<Node>,
    pub nodes_removed: Vec<String>, // names
    pub jobs_upserted: Vec<Job>,
    pub jobs_removed: Vec<String>, // job_ids
    pub partitions_upserted: Vec<Partition>,
    pub partitions_removed: Vec<String>, // names
    pub updated_at: Option<DateTime<Utc>>,
}

impl ClusterState {
    pub fn diff(&self, other: &ClusterState) -> ClusterDiff {
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
    // Build a map of keys to indices
    let mut key_to_index = HashMap::new();
    for (i, item) in list.iter().enumerate() {
        key_to_index.insert(key_fn(item), i);
    }

    // Upsert items (replace if exists, add if new)
    for item in upserted {
        let key = key_fn(&item);
        if let Some(pos) = key_to_index.get(&key) {
            list[*pos] = item;
        } else {
            list.push(item);
        }
    }
}
