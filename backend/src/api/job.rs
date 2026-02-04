use crate::api::{HasId, Partition, Ref, Selectable};
use async_graphql::{Enum, NewType, Object, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, NewType)]
pub struct ClusterJobId(pub i64);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, NewType)]
pub struct JobId(pub Uuid);

#[derive(Enum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Job {
    pub id: JobId,
    pub job_id: ClusterJobId,
    pub name: String,
    pub user: String,
    pub partition: Ref<Partition>,
    pub status: JobStatus,

    pub time_limit: Option<i64>, // in seconds
    pub start_time: Option<DateTime<Utc>>,
    pub submit_time: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub resources: Vec<JobResource>,
    pub allocations: Vec<JobAllocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobPartial {
    pub id: JobId,
    pub job_id: Option<ClusterJobId>,
    pub name: Option<String>,
    pub user: Option<String>,
    // pub partition: PartitionRef,
    pub status: Option<JobStatus>,

    pub time_limit: Option<Option<i64>>, // in seconds
    pub start_time: Option<Option<DateTime<Utc>>>,
    pub submit_time: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    // if Some(), we have the full resources/allocations
    pub resources: Option<Vec<JobResource>>,
    pub allocations: Option<Vec<JobAllocation>>,
}

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobResource {
    // pub resource: ResourceType,
    pub requested: i64,
    pub allocated: i64,
}

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobAllocation {
    // pub node: NodeRef,
    // pub resource: ResourceType,
    pub used: i64,
}

impl HasId for JobPartial {
    type Id = JobId;
    fn id(&self) -> &JobId {
        &self.id
    }
}

impl HasId for Job {
    type Id = JobId;
    fn id(&self) -> &JobId {
        &self.id
    }
}

impl From<Job> for JobPartial {
    fn from(job: Job) -> Self {
        JobPartial {
            id: job.id,
            job_id: Some(job.job_id),
            name: Some(job.name),
            user: Some(job.user),
            status: Some(job.status),
            time_limit: Some(job.time_limit),
            start_time: Some(job.start_time),
            submit_time: Some(job.submit_time),
            updated_at: Some(job.updated_at),
            resources: Some(job.resources),
            allocations: Some(job.allocations),
        }
    }
}

impl Selectable for Job {
    type Selected = JobPartial;
}

#[Object(name = "Job")]
impl Ref<Job> {
    async fn id(&self) -> &JobId {
        match self {
            Ref::Id(id) => id,
            Ref::Selected(selected) => &selected.id,
        }
    }
}
