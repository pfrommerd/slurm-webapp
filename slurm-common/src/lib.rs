use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub state: String, // e.g., "idle", "alloc", "down"
    pub cpus: u32,
    pub real_memory: i64, // in MB, use i64 to be sqlite compatible
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub name: String,
    pub total_nodes: u32,
    pub total_cpus: u32,
    pub state: String, // "UP", "DOWN"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub nodes: Vec<Node>,
    pub jobs: Vec<Job>,
    pub partitions: Vec<Partition>,
    pub updated_at: DateTime<Utc>,
}
