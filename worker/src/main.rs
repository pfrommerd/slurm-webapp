use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use rand::Rng;
use slurm_common::{
    table::Table, ClusterState, Job, JobAllocation, JobId, JobResource, JobStatus, Node, NodeName,
    NodePartition, NodeResource, NodeStatus, Partition, PartitionStatus, ResourceType,
};
use std::time::Duration;
use tokio::time;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "false")]
    /// Run in mock mode (generate fake data)
    mock: bool,

    #[arg(long, default_value = "30")]
    /// Polling interval in seconds
    interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut interval = time::interval(Duration::from_secs(args.interval));
    let mut last_state = ClusterState::default();
    loop {
        interval.tick().await;

        let state = if args.mock {
            Ok(generate_mock_data())
        } else {
            Err("Not implemented")
        };
        match state {
            Ok(state) => {
                let diff = last_state.diff(&state);
                let json = serde_json::to_string(&diff)?;
                println!("{}", json);
                last_state = state;
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn generate_mock_data() -> ClusterState {
    let mut rng = rand::thread_rng();
    let updated_at = Utc::now();

    // Partitions
    let partitions_vec = vec![
        Partition {
            name: "gpu".to_string(),
            status: PartitionStatus::Up,
            total_cpus: 6400,
            total_cpus_alloc: 0,
            total_cpus_idle: 6400,
            total_memory: 2560000,
            total_memory_alloc: 0,
            total_memory_free: 2560000,
            updated_at,
        },
        Partition {
            name: "standard".to_string(),
            status: PartitionStatus::Up,
            total_cpus: 12800,
            total_cpus_alloc: 0,
            total_cpus_idle: 12800,
            total_memory: 5120000,
            total_memory_alloc: 0,
            total_memory_free: 5120000,
            updated_at,
        },
    ];

    let node_states = [
        NodeStatus::Idle,
        NodeStatus::Alloc,
        NodeStatus::Mix,
        NodeStatus::Down,
    ];

    let mut nodes_vec = Vec::new();
    let mut node_resources_vec = Vec::new();
    let mut node_partitions_vec = Vec::new();

    for i in 1..=10 {
        let name = format!("node{:02}", i);
        let node_name = NodeName::new(&name);
        let status = node_states[rng.gen_range(0..node_states.len())];

        nodes_vec.push(Node {
            name: node_name.clone(),
            status,
            cpus: 64,
            cpus_alloc: 0,
            cpus_idle: 64,
            memory: 256000,
            memory_alloc: 0,
            memory_free: 256000,
            partitions: vec![], // This field is technically redundant if we have node_partitions, but let's leave it empty
            updated_at,
        });

        // Add CPU resource
        node_resources_vec.push(NodeResource {
            node: node_name.clone(),
            resource: ResourceType::new("cpu"),
            available: 64,
            total: 64,
        });

        // Add to standard partition
        node_partitions_vec.push(NodePartition {
            node: node_name.clone(),
            partition: "standard".to_string(),
        });

        // Add GPU resource and partition to even nodes
        if i % 2 == 0 {
            node_resources_vec.push(NodeResource {
                node: node_name.clone(),
                resource: ResourceType::new("gpu"),
                available: 4,
                total: 4,
            });
            node_partitions_vec.push(NodePartition {
                node: node_name.clone(),
                partition: "gpu".to_string(),
            });
        }
    }

    let mut jobs_vec = Vec::new();
    let mut job_resources_vec = Vec::new();
    let mut job_allocations_vec = Vec::new();

    for i in 1..=5 {
        let job_id_val = 1000 + i;
        let job_id = JobId::new(job_id_val);
        let status = match rng.gen_range(0..3) {
            0 => JobStatus::Pending,
            1 => JobStatus::Running,
            _ => JobStatus::Completed,
        };

        jobs_vec.push(Job {
            job_id: job_id.clone(),
            user: format!("user{}", rng.gen_range(1..5)),
            partition: "gpu".to_string(),
            status,
            time_limit: Some("12:00:00".to_string()),
            start_time: Some(Utc::now()),
            submit_time: Utc::now(),
            updated_at,
        });

        // Basic resources
        job_resources_vec.push(JobResource {
            job: job_id.clone(),
            resource: ResourceType::new("cpu"),
            requested: rng.gen_range(1..128),
            allocated: if status == JobStatus::Running { 64 } else { 0 },
        });

        if status == JobStatus::Running {
            // Allocate to a random node (simplified)
            let node_idx = rng.gen_range(1..=10);
            let node_name = NodeName::new(&format!("node{:02}", node_idx));
            job_allocations_vec.push(JobAllocation {
                job: job_id.clone(),
                node: node_name,
                resource: ResourceType::new("cpu"),
                used: 64,
            });
        }
    }

    ClusterState {
        partitions: Table::from(partitions_vec),
        nodes: Table::from(nodes_vec),
        node_resources: Table::from(node_resources_vec),
        node_partitions: Table::from(node_partitions_vec),
        jobs: Table::from(jobs_vec),
        job_resources: Table::from(job_resources_vec),
        job_allocations: Table::from(job_allocations_vec),
        updated_at: Some(updated_at),
    }
}
