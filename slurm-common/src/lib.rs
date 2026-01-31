use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "db")]
pub mod db;
pub mod parser;
pub mod scontrol;
pub mod table;

use table::{Keyed, Table};

use crate::table::TableDiff;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ResourceType(String);

impl ResourceType {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeName(String);

impl NodeName {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct JobId(i64);

impl JobId {
    pub fn new(i: i64) -> Self {
        Self(i)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Idle,
    Alloc,
    Mix,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub name: NodeName,
    pub status: NodeStatus,
    // CPU stats
    pub cpus: u32,
    pub cpus_alloc: u32,
    pub cpus_idle: u32,
    // Memory stats
    pub memory: i64,
    pub memory_alloc: i64,
    pub memory_free: i64,
    // Partitions this node belongs to
    pub partitions: Vec<String>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeResource {
    pub node: NodeName,
    pub resource: ResourceType,
    pub available: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodePartition {
    pub node: NodeName,
    pub partition: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub job_id: JobId,
    pub user: String,
    pub partition: String,
    pub status: JobStatus,

    pub time_limit: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub submit_time: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobResource {
    pub job: JobId,
    pub resource: ResourceType,
    pub requested: i64,
    pub allocated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobAllocation {
    pub job: JobId,
    pub node: NodeName,
    pub resource: ResourceType,
    pub used: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PartitionStatus {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Partition {
    pub name: String,

    pub status: PartitionStatus,

    pub total_cpus: u32,
    pub total_cpus_alloc: u32,
    pub total_cpus_idle: u32,

    pub total_memory: i64,
    pub total_memory_alloc: i64,
    pub total_memory_free: i64,

    pub updated_at: DateTime<Utc>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub partitions: Table<Partition>,
    pub nodes: Table<Node>,
    pub jobs: Table<Job>,
    pub node_resources: Table<NodeResource>,
    pub node_partitions: Table<NodePartition>,
    pub job_resources: Table<JobResource>,
    pub job_allocations: Table<JobAllocation>,
    // The time this state was last refreshed
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterDiff {
    pub partitions: TableDiff<Partition, String>,
    pub nodes: TableDiff<Node, NodeName>,
    pub jobs: TableDiff<Job, JobId>,
    pub node_resources: TableDiff<NodeResource, (NodeName, ResourceType)>,
    pub node_partitions: TableDiff<NodePartition, (NodeName, String)>,
    pub job_resources: TableDiff<JobResource, (JobId, ResourceType)>,
    pub job_allocations: TableDiff<JobAllocation, (JobId, NodeName, ResourceType)>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ClusterState {
    pub fn diff(&self, other: &ClusterState) -> ClusterDiff {
        ClusterDiff {
            partitions: self.partitions.diff(&other.partitions),
            nodes: self.nodes.diff(&other.nodes),
            jobs: self.jobs.diff(&other.jobs),
            node_resources: self.node_resources.diff(&other.node_resources),
            node_partitions: self.node_partitions.diff(&other.node_partitions),
            job_resources: self.job_resources.diff(&other.job_resources),
            job_allocations: self.job_allocations.diff(&other.job_allocations),
            updated_at: other.updated_at,
        }
    }

    pub fn apply(&mut self, diff: ClusterDiff) {
        self.updated_at = diff.updated_at;
    }
}

// Implement the Keyed trait for the different types

impl Keyed for Partition {
    type Key = String;
    type KeyRef<'s>
        = &'s str
    where
        Self: 's;

    fn key(&self) -> Self::KeyRef<'_> {
        &self.name
    }

    fn clone_key(r: Self::KeyRef<'_>) -> Self::Key {
        r.to_string()
    }
}

impl Keyed for Node {
    type Key = NodeName;
    type KeyRef<'s>
        = &'s NodeName
    where
        Self: 's;

    fn key(&self) -> Self::KeyRef<'_> {
        &self.name
    }

    fn clone_key(r: Self::KeyRef<'_>) -> Self::Key {
        r.clone()
    }
}

impl Keyed for NodeResource {
    type Key = (NodeName, ResourceType);
    type KeyRef<'s>
        = (&'s NodeName, &'s ResourceType)
    where
        Self: 's;

    fn key(&self) -> Self::KeyRef<'_> {
        (&self.node, &self.resource)
    }

    fn clone_key(r: Self::KeyRef<'_>) -> Self::Key {
        (r.0.clone(), r.1.clone())
    }
}

impl Keyed for NodePartition {
    type Key = (NodeName, String);
    type KeyRef<'s>
        = (&'s NodeName, &'s str)
    where
        Self: 's;

    fn key(&self) -> Self::KeyRef<'_> {
        (&self.node, &self.partition)
    }

    fn clone_key(r: Self::KeyRef<'_>) -> Self::Key {
        (r.0.clone(), r.1.to_string())
    }
}

impl Keyed for Job {
    type Key = JobId;
    type KeyRef<'s>
        = &'s JobId
    where
        Self: 's;

    fn key(&self) -> Self::KeyRef<'_> {
        &self.job_id
    }

    fn clone_key(r: Self::KeyRef<'_>) -> Self::Key {
        r.clone()
    }
}

impl Keyed for JobResource {
    type Key = (JobId, ResourceType);
    type KeyRef<'s>
        = (&'s JobId, &'s ResourceType)
    where
        Self: 's;

    fn key(&self) -> Self::KeyRef<'_> {
        (&self.job, &self.resource)
    }

    fn clone_key(r: Self::KeyRef<'_>) -> Self::Key {
        (r.0.clone(), r.1.clone())
    }
}

impl Keyed for JobAllocation {
    type Key = (JobId, NodeName, ResourceType);
    type KeyRef<'s>
        = (&'s JobId, &'s NodeName, &'s ResourceType)
    where
        Self: 's;

    fn key(&self) -> Self::KeyRef<'_> {
        (&self.job, &self.node, &self.resource)
    }

    fn clone_key(r: Self::KeyRef<'_>) -> Self::Key {
        (r.0.clone(), r.1.clone(), r.2.clone())
    }
}
