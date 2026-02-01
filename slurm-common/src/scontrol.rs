use anyhow::Result;
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

use crate::{
    table::Table, Job, JobAllocation, JobResource, Node, NodeName, NodePartition, NodeResource,
    Partition, PartitionStatus, ResourceType,
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

pub async fn nodes() -> Result<(Table<Node>, Table<NodeResource>, Table<NodePartition>)> {
    let output = tokio::process::Command::new("scontrol")
        .arg("show")
        .arg("nodes")
        .output()
        .await?;
    let output = String::from_utf8(output.stdout)?;
    let node_infos: Vec<NodeInfo> = crate::parser::from_str(&output).unwrap_or_default();

    let mut nodes = Table::new();
    let mut resources = Table::new();
    let mut partitions = Table::new();
    let updated_at = chrono::Utc::now();

    for info in node_infos {
        let name = crate::NodeName(info.name.to_string());

        // Map scontrol state to our NodeStatus
        let status = match info.state {
            NodeStateInfo::Idle => crate::NodeStatus::Idle,
            NodeStateInfo::Allocated => crate::NodeStatus::Alloc,
            NodeStateInfo::Mix => crate::NodeStatus::Mix,
            NodeStateInfo::Down => crate::NodeStatus::Down,
            NodeStateInfo::Unknown => crate::NodeStatus::Unknown,
        };

        // Node
        nodes.insert(Node {
            name: name.clone(),
            status,
            cpus: info.cpus,
            cpus_alloc: info.cpu_alloc,
            cpus_idle: info.cpus.saturating_sub(info.cpu_alloc),
            memory: info.real_memory as i64,
            memory_alloc: info.alloc_mem as i64,
            memory_free: info.free_mem.unwrap_or(0) as i64,
            partitions: info.partitions.iter().map(|s| s.to_string()).collect(),
            updated_at,
        });

        // Node Partitions
        for part_name in info.partitions {
            partitions.insert(NodePartition {
                node: name.clone(),
                partition: part_name.to_string(),
            });
        }

        // Node Resources (CfgTRES vs AllocTRES)
        // We'll iterate over CfgTRES for 'total' and compare with AllocTRES for 'available'
        // But AllocTRES only shows allocated. Available = Total - Allocated.
        for (res_name, total_qty) in info.resources {
            let total = total_qty.0 as u64;
            let allocated = info.allocated.get(res_name).map(|q| q.0).unwrap_or(0);
            let available = total.saturating_sub(allocated as u64);

            resources.insert(NodeResource {
                node: name.clone(),
                resource: crate::ResourceType(res_name.to_string()),
                total,
                available,
            });
        }
    }

    Ok((nodes, resources, partitions))
}

pub async fn partitions() -> Result<Table<Partition>> {
    let output = tokio::process::Command::new("scontrol")
        .arg("show")
        .arg("partitions")
        .output()
        .await?;
    let output = String::from_utf8(output.stdout).unwrap();
    let partitions: Vec<PartitionInfo> = crate::parser::from_str(&output).unwrap();
    let mut table = Table::new();
    for info in partitions {
        let status = match info.state {
            NodeStateInfo::Idle => PartitionStatus::Up,
            NodeStateInfo::Allocated => PartitionStatus::Up,
            NodeStateInfo::Mix => PartitionStatus::Up,
            NodeStateInfo::Down => PartitionStatus::Down,
            NodeStateInfo::Unknown => PartitionStatus::Down,
        };
        table.insert(Partition {
            name: info.name.to_string(),
            status,
            access_qos: info.allow_qos.map(|s| s.to_string()),
            resource_qos: info.qos.map(|s| s.to_string()),
            updated_at: chrono::Utc::now(),
        });
    }
    Ok(table)
}

pub async fn jobs() -> Result<(Table<Job>, Table<JobAllocation>, Table<JobResource>)> {
    let output = tokio::process::Command::new("scontrol")
        .arg("show")
        .arg("jobs")
        .arg("--details")
        .output()
        .await?;
    let output = String::from_utf8(output.stdout)?;
    let job_infos: Vec<JobInfo> = crate::parser::from_str(&output).unwrap_or_default();

    let mut jobs = Table::new();
    let mut allocations = Table::new();
    let mut resources = Table::new();
    let updated_at = Utc::now();

    for info in job_infos {
        let job_id = crate::JobId::new(info.job_id as i64);

        // Status mapping
        let status = match info.state {
            JobStateInfo::Running => crate::JobStatus::Running,
            JobStateInfo::Pending => crate::JobStatus::Pending,
            JobStateInfo::Completed => crate::JobStatus::Completed,
            JobStateInfo::Failed => crate::JobStatus::Failed,
            JobStateInfo::Unknown => crate::JobStatus::Unknown,
        };

        // Parse user (remove uid part if present, e.g. "user(123)")
        let user_str = info.user.split('(').next().unwrap_or(info.user);

        let submit_time = parse_slurm_time(info.submit_time).unwrap_or(updated_at);
        let start_time = info.start_time.and_then(parse_slurm_time);
        let time_limit = info.time_limit.and_then(parse_slurm_duration);

        jobs.insert(Job {
            job_id: job_id.clone(),
            name: info.name.to_string(),
            user: user_str.to_string(),
            partition: info.partition.to_string(),
            status,
            time_limit,
            start_time,
            submit_time,
            updated_at,
        });

        // Resources
        // We'll collect all resource keys from both ReqTRES and AllocTRES
        let mut res_keys = HashSet::new();
        if let Some(req) = &info.req_res {
            for k in req.keys() {
                res_keys.insert(k.to_string());
            }
        }
        if let Some(alloc) = &info.alloc_res {
            for k in alloc.keys() {
                res_keys.insert(k.to_string());
            }
        }

        for res_key in res_keys {
            let requested = info
                .req_res
                .as_ref()
                .and_then(|m| m.get(res_key.as_str()))
                .map(|q| q.clone().into())
                .unwrap_or(0);
            let allocated = info
                .alloc_res
                .as_ref()
                .and_then(|m| m.get(res_key.as_str()))
                .map(|q| q.clone().into())
                .unwrap_or(0);

            if requested > 0 || allocated > 0 {
                resources.insert(JobResource {
                    job: job_id.clone(),
                    resource: ResourceType::new(&res_key),
                    requested,
                    allocated,
                });
            }
        }

        // Job Allocation (only if single node for now)
        if info.node_list.len() == 1 {
            if let Some(alloc) = &info.alloc_res {
                let node_name = NodeName::new(&info.node_list[0]);
                for (res, qty) in alloc {
                    allocations.insert(JobAllocation {
                        job: job_id.clone(),
                        node: node_name.clone(),
                        resource: ResourceType::new(res),
                        used: qty.clone().into(),
                    });
                }
            }
        }
    }
    Ok((jobs, allocations, resources))
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
