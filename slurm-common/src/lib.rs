use async_graphql::{Enum, NewType, Object, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod parser;
pub mod scontrol;
pub mod table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, NewType)]
pub struct ResourceType(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, NewType)]
pub struct ClusterJobId(pub i64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, NewType)]
pub struct NodeName(pub String);

#[rustfmt::skip]
impl AsRef<str> for ResourceType {
    fn as_ref(&self) -> &str { &self.0 }
}
#[rustfmt::skip]
impl AsRef<str> for NodeName {
    fn as_ref(&self) -> &str { &self.0 }
}

#[derive(Enum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Unknown,
}

#[derive(Enum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Idle,
    Alloc,
    Mix,
    Down,
    Unknown,
}

#[derive(Enum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PartitionStatus {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Job {
    pub id: Uuid,
    pub job_id: ClusterJobId,
    pub name: String,
    pub user: String,
    pub partition: String,
    pub status: JobStatus,

    pub time_limit: Option<i64>, // in seconds
    pub start_time: Option<DateTime<Utc>>,
    pub submit_time: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub resources: Vec<JobResource>,
    pub allocations: Vec<JobAllocation>,
}

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobResource {
    pub resource: ResourceType,
    pub requested: i64,
    pub allocated: i64,
}

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobAllocation {
    pub node: NodeRef,
    pub resource: ResourceType,
    pub used: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobRef {
    Id(Uuid),
    Job(Job),
}

#[Object(name = "Job")]
impl JobRef {
    async fn id(&self) -> &Uuid {
        match self {
            JobRef::Id(id) => id,
            JobRef::Job(job) => &job.id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub id: Uuid,
    pub name: NodeName,
    pub status: NodeStatus,
    pub partitions: Vec<PartitionRef>,
    pub resources: Vec<ResourceAvailability>,
    pub updated_at: DateTime<Utc>,
}

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceAvailability {
    pub resource: ResourceType,
    pub available: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeRef {
    Id(Uuid),
    Node(Node),
}

#[Object(name = "Node")]
impl NodeRef {
    async fn id(&self) -> &Uuid {
        match self {
            NodeRef::Id(id) => id,
            NodeRef::Node(node) => &node.id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Partition {
    pub id: Uuid,
    pub name: String,
    pub status: PartitionStatus,
    // The QoS governing this partition
    pub access_qos: Option<String>,
    pub resource_qos: Option<String>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PartitionRef {
    Id(Uuid),
    Partition(Partition),
}

#[Object(name = "Partition")]
impl PartitionRef {
    async fn id(&self) -> &Uuid {
        match self {
            PartitionRef::Id(id) => id,
            PartitionRef::Partition(partition) => &partition.id,
        }
    }
}
