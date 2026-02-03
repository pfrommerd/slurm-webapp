use anyhow::Result;
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

use crate::table::{Table, TableDiff};
use crate::{
    Job, JobAllocation, JobResource, Node, NodeName, Partition, PartitionStatus, ResourceType,
};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum NodeStateInfo {
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

#[derive(Debug, Clone, Deserialize)]
pub struct NodeInfo<'src> {
    #[serde(rename = "NodeName")]
    pub name: &'src str,
    #[serde(rename = "State")]
    pub state: NodeStateInfo,
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
    pub resources: BTreeMap<&'src str, ResourceQuantity>,
    #[serde(rename = "AllocTRES")]
    pub allocated: BTreeMap<&'src str, ResourceQuantity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionInfo<'src> {
    #[serde(rename = "PartitionName")]
    pub name: &'src str,
    #[serde(rename = "State")]
    pub state: NodeStateInfo,
    #[serde(rename = "AllowQos")]
    pub allow_qos: Option<&'src str>,
    #[serde(rename = "QoS")]
    pub qos: Option<&'src str>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum JobStateInfo {
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

#[derive(Debug, Clone, Deserialize)]
pub struct JobInfo<'src> {
    #[serde(rename = "JobId")]
    pub job_id: u64,
    #[serde(rename = "JobName")]
    pub name: &'src str,
    #[serde(rename = "Partition")]
    pub partition: &'src str,
    #[serde(rename = "UserId")]
    pub user: &'src str,
    #[serde(rename = "JobState")]
    pub state: JobStateInfo,
    #[serde(rename = "NumCPUs")]
    pub num_cpus: u32,
    #[serde(rename = "NumNodes")]
    pub num_nodes: String, // sometimes weird, like 2-2 or 1-1
    #[serde(rename = "NodeList")]
    pub node_list: Vec<String>,
    #[serde(rename = "ReqTRES")]
    pub req_res: Option<BTreeMap<&'src str, ResourceQuantity>>,
    #[serde(rename = "AllocTRES")]
    pub alloc_res: Option<BTreeMap<&'src str, ResourceQuantity>>,
    #[serde(rename = "SubmitTime")]
    pub submit_time: &'src str,
    #[serde(rename = "StartTime")]
    pub start_time: Option<&'src str>,
    #[serde(rename = "TimeLimit")]
    pub time_limit: Option<&'src str>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterState {
    pub nodes: Table<Node>,
    pub partitions: Table<Partition>,
    pub jobs: Table<Job>,
}

impl ClusterState {
    pub fn apply(&mut self, diff: ClusterDiff) {
        self.nodes.apply(diff.nodes);
        self.partitions.apply(diff.partitions);
        self.jobs.apply(diff.jobs);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterDiff {
    pub nodes: TableDiff<Node>,
    pub partitions: TableDiff<Partition>,
    pub jobs: TableDiff<Job>,
}

pub async fn state(current: &ClusterState) -> Result<ClusterDiff> {
    // gather all and await all
    let (nodes, partitions, jobs) = tokio::join!(
        nodes(&current.nodes),
        partitions(&current.partitions),
        jobs(&current.jobs)
    );
    let nodes = nodes?;
    let partitions = partitions?;
    let jobs = jobs?;
    Ok(ClusterDiff {
        nodes,
        partitions,
        jobs,
    })
}

pub async fn nodes(current: &Table<Node>) -> Result<TableDiff<Node>> {
    let output = tokio::process::Command::new("scontrol")
        .arg("show")
        .arg("nodes")
        .output()
        .await?;
    let output = String::from_utf8(output.stdout)?;
    let node_infos: Vec<NodeInfo> = crate::parser::from_str(&output).unwrap_or_default();
    todo!()
}

pub async fn partitions(current: &Table<Partition>) -> Result<TableDiff<Partition>> {
    let output = tokio::process::Command::new("scontrol")
        .arg("show")
        .arg("partitions")
        .output()
        .await?;
    let output = String::from_utf8(output.stdout).unwrap();
    let partitions: Vec<PartitionInfo> = crate::parser::from_str(&output).unwrap();
    todo!()
}

pub async fn jobs(current: &Table<Job>) -> Result<TableDiff<Job>> {
    let output = tokio::process::Command::new("scontrol")
        .arg("show")
        .arg("jobs")
        .arg("--details")
        .output()
        .await?;
    let output = String::from_utf8(output.stdout)?;
    let job_infos: Vec<JobInfo> = crate::parser::from_str(&output).unwrap_or_default();
    todo!()
}

fn parse_slurm_time(s: &str) -> Option<chrono::DateTime<Utc>> {
    if s == "N/A" || s == "None" {
        return None;
    }
    // format: 2026-01-31T12:44:31
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|dt| Utc.from_utc_datetime(&dt))
}

fn parse_slurm_duration(s: &str) -> Option<i64> {
    if s == "UNLIMITED" || s == "N/A" || s == "None" {
        return None;
    }
    // Formats: MM:SS, HH:MM:SS, D-HH:MM:SS
    let parts: Vec<&str> = s.split('-').collect();
    let (days, time_part) = if parts.len() == 2 {
        (parts[0].parse::<i64>().ok()?, parts[1])
    } else {
        (0, parts[0])
    };

    let time_split: Vec<&str> = time_part.split(':').collect();
    let seconds = if time_split.len() == 3 {
        let h: i64 = time_split[0].parse().ok()?;
        let m: i64 = time_split[1].parse().ok()?;
        let s: i64 = time_split[2].parse().ok()?;
        h * 3600 + m * 60 + s
    } else if time_split.len() == 2 {
        let m: i64 = time_split[0].parse().ok()?;
        let s: i64 = time_split[1].parse().ok()?;
        m * 60 + s
    } else {
        return None;
    };

    Some(days * 24 * 3600 + seconds)
}

// Will handle parsing memory M and G suffixes
#[derive(Debug, Clone)]
pub struct ResourceQuantity(i64);

impl Into<i64> for ResourceQuantity {
    fn into(self) -> i64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ResourceQuantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResVisitor;

        impl<'de> serde::de::Visitor<'de> for ResVisitor {
            type Value = ResourceQuantity;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string like '100M' or '1G' or a raw number")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut value = v.trim();
                let mut multiplier = 1;
                if value.ends_with('M') {
                    multiplier = 1000 * 1000;
                    value = &value[..value.len() - 1];
                } else if value.ends_with('G') {
                    multiplier = 1000 * 1000 * 1000;
                    value = &value[..value.len() - 1];
                }
                Ok(ResourceQuantity(
                    (value.parse::<f64>().map_err(|_| {
                        E::custom(format!(
                            "Invalid resource quantity: {} in specifier {}",
                            value, v
                        ))
                    })? * multiplier as f64) as i64,
                ))
            }
        }
        deserializer.deserialize_str(ResVisitor)
    }
}
