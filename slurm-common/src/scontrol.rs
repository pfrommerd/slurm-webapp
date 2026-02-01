use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::{
    table::Table, Job, JobAllocation, JobResource, Node, NodePartition, NodeResource, Partition,
    PartitionStatus,
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
    let output = String::from_utf8(output.stdout).unwrap();
    let jobs: Vec<JobInfo> = crate::parser::from_str(&output).unwrap();
    eprintln!("{:?}", jobs);
    Ok((Table::new(), Table::new(), Table::new()))
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
