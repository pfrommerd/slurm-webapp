pub mod parser;

#[cfg(feature = "ssh")]
pub mod ssh;
pub mod util;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::util::{SlurmDuration, SlurmQuantity};

macro_rules! diff_struct {
    ($struct:ident, $self:ident, $other:ident, $($field:ident),* $(,)?) => {
        $struct {
            $(
                $field: if $self.$field != $other.$field {
                    Some($other.$field.clone())
                } else {
                    None
                }
            ),*
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum PartitionState {
    #[serde(rename = "UP")]
    Up,
    #[serde(rename = "DOWN")]
    Down,
    #[serde(rename = "UNKNOWN", other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SlurmPartition<'src> {
    #[serde(rename = "PartitionName")]
    pub name: &'src str,
    #[serde(rename = "State")]
    pub state: PartitionState,
    #[serde(rename = "AllowQos")]
    pub allow_qos: Option<&'src str>,
    #[serde(rename = "QoS")]
    pub qos: Option<&'src str>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SlurmPartitionDiff<'src> {
    pub name: Option<&'src str>,
    pub state: Option<PartitionState>,
    pub allow_qos: Option<Option<&'src str>>,
    pub qos: Option<Option<&'src str>>,
}

impl<'src> SlurmPartition<'src> {
    pub fn diff(&self, other: &SlurmPartition<'src>) -> SlurmPartitionDiff<'src> {
        diff_struct!(SlurmPartitionDiff, self, other, name, state, allow_qos, qos)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum JobState {
    #[serde(rename = "RUNNING")]
    Running,
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "UNKNOWN", other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SlurmJob<'src> {
    #[serde(rename = "JobId")]
    pub job_id: u64,
    #[serde(rename = "JobName")]
    pub name: &'src str,
    #[serde(rename = "Partition")]
    pub partition: &'src str,
    #[serde(rename = "UserId")]
    pub user: &'src str,
    #[serde(rename = "JobState")]
    pub state: JobState,
    #[serde(rename = "NumCPUs")]
    pub num_cpus: u32,
    #[serde(rename = "NumNodes")]
    pub num_nodes: &'src str, // sometimes weird, like 2-2 or 1-1
    #[serde(rename = "NodeList")]
    pub node_list: Vec<&'src str>,
    #[serde(rename = "ReqTRES")]
    pub req_res: Option<BTreeMap<&'src str, SlurmQuantity>>,
    #[serde(rename = "AllocTRES")]
    pub alloc_res: Option<BTreeMap<&'src str, SlurmQuantity>>,
    #[serde(rename = "SubmitTime")]
    pub submit_time: DateTime<Utc>,
    #[serde(rename = "StartTime")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(rename = "TimeLimit")]
    pub time_limit: Option<SlurmDuration>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SlurmJobDiff<'src> {
    pub job_id: Option<u64>,
    pub name: Option<&'src str>,
    pub partition: Option<&'src str>,
    pub user: Option<&'src str>,
    pub state: Option<JobState>,
    pub num_cpus: Option<u32>,
    pub num_nodes: Option<&'src str>,
    pub node_list: Option<Vec<&'src str>>,
    pub req_res: Option<Option<BTreeMap<&'src str, SlurmQuantity>>>,
    pub alloc_res: Option<Option<BTreeMap<&'src str, SlurmQuantity>>>,
    pub submit_time: Option<DateTime<Utc>>,
    pub start_time: Option<Option<DateTime<Utc>>>,
    pub time_limit: Option<Option<SlurmDuration>>,
}

impl<'src> SlurmJob<'src> {
    pub fn diff(&self, other: &SlurmJob<'src>) -> SlurmJobDiff<'src> {
        diff_struct!(
            SlurmJobDiff,
            self,
            other,
            job_id,
            name,
            partition,
            user,
            state,
            num_cpus,
            num_nodes,
            node_list,
            req_res,
            alloc_res,
            submit_time,
            start_time,
            time_limit
        )
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum NodeState {
    #[serde(rename = "IDLE")]
    Idle,
    #[serde(rename = "ALLOCATED")]
    Allocated,
    #[serde(rename = "MIX")]
    Mix,
    #[serde(rename = "DOWN")]
    Down,
    #[serde(rename = "UNKNOWN", other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SlurmNode<'src> {
    #[serde(rename = "NodeName")]
    pub name: &'src str,
    #[serde(rename = "State")]
    pub state: NodeState,
    #[serde(rename = "CPUAlloc")]
    pub cpu_alloc: u32,
    #[serde(rename = "CPUTot")]
    pub cpus: u32,
    #[serde(rename = "RealMemory")]
    pub real_memory: u32,
    #[serde(rename = "AllocMem")]
    pub alloc_mem: u32,
    #[serde(rename = "FreeMem")]
    pub free_mem: Option<u32>, // May be N/A if node is DOWN

    #[serde(rename = "Partitions")]
    pub partitions: Vec<&'src str>,
    #[serde(rename = "CfgTRES")]
    pub resources: BTreeMap<&'src str, SlurmQuantity>,
    #[serde(rename = "AllocTRES")]
    pub allocated: BTreeMap<&'src str, SlurmQuantity>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SlurmNodeDiff<'src> {
    pub name: Option<&'src str>,
    pub state: Option<NodeState>,
    pub cpu_alloc: Option<u32>,
    pub cpus: Option<u32>,
    pub real_memory: Option<u32>,
    pub alloc_mem: Option<u32>,
    pub free_mem: Option<Option<u32>>,
    pub partitions: Option<Vec<&'src str>>,
    pub resources: Option<BTreeMap<&'src str, SlurmQuantity>>,
    pub allocated: Option<BTreeMap<&'src str, SlurmQuantity>>,
}

impl<'src> SlurmNode<'src> {
    pub fn diff(&self, other: &SlurmNode<'src>) -> SlurmNodeDiff<'src> {
        diff_struct!(
            SlurmNodeDiff,
            self,
            other,
            name,
            state,
            cpu_alloc,
            cpus,
            real_memory,
            alloc_mem,
            free_mem,
            partitions,
            resources,
            allocated
        )
    }
}

pub struct RawSlurmInfo {
    pub nodes: String,
    pub partitions: String,
    pub jobs: String,
}

impl RawSlurmInfo {
    pub fn empty() -> Self {
        Self {
            nodes: String::new(),
            partitions: String::new(),
            jobs: String::new(),
        }
    }
    pub async fn from_local_system() -> Result<Self> {
        let (nodes, jobs, partitions) = tokio::join!(
            tokio::process::Command::new("scontrol")
                .arg("show")
                .arg("nodes")
                .output(),
            tokio::process::Command::new("scontrol")
                .arg("show")
                .arg("jobs")
                .arg("--details")
                .output(),
            tokio::process::Command::new("scontrol")
                .arg("show")
                .arg("partitions")
                .output(),
        );
        let nodes = String::from_utf8(nodes?.stdout)?;
        let jobs = String::from_utf8(jobs?.stdout)?;
        let partitions = String::from_utf8(partitions?.stdout)?;
        Ok(Self {
            nodes,
            jobs,
            partitions,
        })
    }

    pub fn parse<'src>(&'src self) -> Result<SlurmCluster<'src>> {
        let nodes_vec: Vec<SlurmNode> = parser::from_str(&self.nodes)?;
        let partitions_vec: Vec<SlurmPartition> = parser::from_str(&self.partitions)?;
        let jobs_vec: Vec<SlurmJob> = parser::from_str(&self.jobs)?;

        let mut nodes = BTreeMap::new();
        for node in nodes_vec {
            nodes.insert(node.name, node);
        }

        let mut partitions = BTreeMap::new();
        for part in partitions_vec {
            partitions.insert(part.name, part);
        }

        let mut jobs = BTreeMap::new();
        for job in jobs_vec {
            jobs.insert(job.name, job);
        }

        Ok(SlurmCluster {
            nodes,
            partitions,
            jobs,
        })
    }
}

pub struct SlurmInfo {
    raw: Box<RawSlurmInfo>,
    parsed: SlurmCluster<'static>,
}

impl SlurmInfo {
    pub fn new(raw: RawSlurmInfo) -> Result<Self> {
        let boxed = Box::new(raw);
        // SAFETY: We are transmuting the lifetime of the boxed, raw data.
        // The we always constraint the return lifetime to the parsed struct
        // to the lifetime of the SlurmInfo struct, so this is okay!
        let parsed = raw.parse()?;
        Ok(Self {
            raw: Box::new(raw),
            parsed,
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlurmCluster<'src> {
    #[serde(borrow)]
    pub nodes: BTreeMap<&'src str, SlurmNode<'src>>,
    #[serde(borrow)]
    pub partitions: BTreeMap<&'src str, SlurmPartition<'src>>,
    #[serde(borrow)]
    pub jobs: BTreeMap<&'src str, SlurmJob<'src>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlurmClusterDiff<'src> {
    #[serde(borrow)]
    pub new_nodes: BTreeMap<&'src str, SlurmNode<'src>>,
    #[serde(borrow)]
    pub changed_nodes: BTreeMap<&'src str, SlurmNodeDiff<'src>>,
    #[serde(borrow)]
    pub removed_nodes: BTreeSet<&'src str>,

    #[serde(borrow)]
    pub new_partitions: BTreeMap<&'src str, SlurmPartition<'src>>,
    #[serde(borrow)]
    pub changed_partitions: BTreeMap<&'src str, SlurmPartitionDiff<'src>>,
    #[serde(borrow)]
    pub removed_partitions: BTreeSet<&'src str>,

    #[serde(borrow)]
    pub new_jobs: BTreeMap<&'src str, SlurmJob<'src>>,
    #[serde(borrow)]
    pub changed_jobs: BTreeMap<&'src str, SlurmJobDiff<'src>>,
    #[serde(borrow)]
    pub removed_jobs: BTreeSet<&'src str>,
}

impl<'src> SlurmCluster<'src> {
    pub fn diff(&self, other: &SlurmCluster<'src>) -> SlurmClusterDiff<'src> {
        // Nodes
        let mut new_nodes = BTreeMap::new();
        let mut changed_nodes = BTreeMap::new();
        let mut removed_nodes = BTreeSet::new();

        for (name, node) in &other.nodes {
            if !self.nodes.contains_key(name) {
                new_nodes.insert(*name, node.clone());
            } else {
                let diff = self.nodes.get(name).unwrap().diff(node);
                if self.nodes.get(name).unwrap() != node {
                    changed_nodes.insert(*name, diff);
                }
            }
        }
        for name in self.nodes.keys() {
            if !other.nodes.contains_key(name) {
                removed_nodes.insert(*name);
            }
        }

        // Partitions
        let mut new_partitions = BTreeMap::new();
        let mut changed_partitions = BTreeMap::new();
        let mut removed_partitions = BTreeSet::new();

        for (name, part) in &other.partitions {
            if !self.partitions.contains_key(name) {
                new_partitions.insert(*name, part.clone());
            } else {
                let diff = self.partitions.get(name).unwrap().diff(part);
                if self.partitions.get(name).unwrap() != part {
                    changed_partitions.insert(*name, diff);
                }
            }
        }
        for name in self.partitions.keys() {
            if !other.partitions.contains_key(name) {
                removed_partitions.insert(*name);
            }
        }

        // Jobs
        let mut new_jobs = BTreeMap::new();
        let mut changed_jobs = BTreeMap::new();
        let mut removed_jobs = BTreeSet::new();

        for (name, job) in &other.jobs {
            if !self.jobs.contains_key(name) {
                new_jobs.insert(*name, job.clone());
            } else {
                let diff = self.jobs.get(name).unwrap().diff(job);
                if self.jobs.get(name).unwrap() != job {
                    changed_jobs.insert(*name, diff);
                }
            }
        }
        for name in self.jobs.keys() {
            if !other.jobs.contains_key(name) {
                removed_jobs.insert(*name);
            }
        }

        SlurmClusterDiff {
            new_nodes,
            changed_nodes,
            removed_nodes,
            new_partitions,
            changed_partitions,
            removed_partitions,
            new_jobs,
            changed_jobs,
            removed_jobs,
        }
    }

    pub fn apply(&mut self, diff: SlurmClusterDiff<'src>) {
        // Nodes
        for (name, node) in diff.new_nodes {
            self.nodes.insert(name, node);
        }
        for (name, node_diff) in diff.changed_nodes {
            if let Some(node) = self.nodes.get_mut(name) {
                // Apply node diff - field by field
                if let Some(state) = node_diff.state {
                    node.state = state;
                }
                if let Some(cpu_alloc) = node_diff.cpu_alloc {
                    node.cpu_alloc = cpu_alloc;
                }
                if let Some(cpus) = node_diff.cpus {
                    node.cpus = cpus;
                }
                if let Some(real_memory) = node_diff.real_memory {
                    node.real_memory = real_memory;
                }
                if let Some(alloc_mem) = node_diff.alloc_mem {
                    node.alloc_mem = alloc_mem;
                }
                if let Some(free_mem) = node_diff.free_mem {
                    node.free_mem = free_mem;
                }
                if let Some(partitions) = node_diff.partitions {
                    node.partitions = partitions;
                }
                if let Some(resources) = node_diff.resources {
                    node.resources = resources;
                }
                if let Some(allocated) = node_diff.allocated {
                    node.allocated = allocated;
                }
            }
        }
        for name in diff.removed_nodes {
            self.nodes.remove(name);
        }

        // Partitions
        for (name, part) in diff.new_partitions {
            self.partitions.insert(name, part);
        }
        for (name, part_diff) in diff.changed_partitions {
            if let Some(part) = self.partitions.get_mut(name) {
                if let Some(state) = part_diff.state {
                    part.state = state;
                }
                if let Some(allow_qos) = part_diff.allow_qos {
                    part.allow_qos = allow_qos;
                }
                if let Some(qos) = part_diff.qos {
                    part.qos = qos;
                }
            }
        }
        for name in diff.removed_partitions {
            self.partitions.remove(name);
        }

        // Jobs
        for (name, job) in diff.new_jobs {
            self.jobs.insert(name, job);
        }
        for (name, job_diff) in diff.changed_jobs {
            if let Some(job) = self.jobs.get_mut(name) {
                if let Some(partition) = job_diff.partition {
                    job.partition = partition;
                }
                if let Some(user) = job_diff.user {
                    job.user = user;
                }
                if let Some(state) = job_diff.state {
                    job.state = state;
                }
                if let Some(num_cpus) = job_diff.num_cpus {
                    job.num_cpus = num_cpus;
                }
                if let Some(num_nodes) = job_diff.num_nodes {
                    job.num_nodes = num_nodes;
                }
                if let Some(node_list) = job_diff.node_list {
                    job.node_list = node_list;
                }
                if let Some(req_res) = job_diff.req_res {
                    job.req_res = req_res;
                }
                if let Some(alloc_res) = job_diff.alloc_res {
                    job.alloc_res = alloc_res;
                }
                if let Some(submit_time) = job_diff.submit_time {
                    job.submit_time = submit_time;
                }
                if let Some(start_time) = job_diff.start_time {
                    job.start_time = start_time;
                }
                if let Some(time_limit) = job_diff.time_limit {
                    job.time_limit = time_limit;
                }
            }
        }
        for name in diff.removed_jobs {
            self.jobs.remove(name);
        }
    }
}
