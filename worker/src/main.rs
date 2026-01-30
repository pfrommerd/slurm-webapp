use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use rand::Rng;
use slurm_common::{ClusterStatus, Job, Node, Partition, Resource};
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
    let mut last_status = ClusterStatus::default();
    loop {
        interval.tick().await;

        let status = if args.mock {
            generate_mock_data()
        } else {
            // TODO: partial implementation of real SLURM commands
            // For now, we fall back to mock or empty
            eprintln!("Real SLURM data collection not fully implemented yet, sending mock data anyway for testing.");
            generate_mock_data()
        };

        let diff = last_status.diff(&status);
        let json = serde_json::to_string(&diff)?;
        println!("{}", json);
        last_status = status;
    }
}

fn generate_mock_data() -> ClusterStatus {
    let mut rng = rand::thread_rng();
    let updated_at = Utc::now();

    let node_states = ["idle", "alloc", "mix", "down"];
    let mut nodes = Vec::new();
    for i in 1..=10 {
        let mut resources = std::collections::HashMap::new();
        // Add CPU resource
        resources.insert(
            "cpu".to_string(),
            Resource {
                res_id: "cpu".to_string(),
                total: 64,
                allocated: rng.gen_range(0..=64),
            },
        );

        // Add GPU resource (maybe)
        if i % 2 == 0 {
            resources.insert(
                "gpu".to_string(),
                Resource {
                    res_id: "gpu".to_string(),
                    total: 4,
                    allocated: rng.gen_range(0..=4),
                },
            );
        }

        nodes.push(Node {
            name: format!("node{:02}", i),
            state: node_states[rng.gen_range(0..node_states.len())].to_string(),
            cpus: 64,
            real_memory: 256000,
            resources,
        });
    }

    let mut jobs = Vec::new();
    for i in 1..=5 {
        jobs.push(Job {
            job_id: format!("{}", 1000 + i),
            user: format!("user{}", rng.gen_range(1..5)),
            partition: "gpu".to_string(),
            state: match rng.gen_range(0..3) {
                0 => slurm_common::JobState::PENDING,
                1 => slurm_common::JobState::RUNNING,
                _ => slurm_common::JobState::COMPLETED,
            },
            num_nodes: rng.gen_range(1..4),
            num_cpus: rng.gen_range(1..128),
            time_limit: Some("12:00:00".to_string()),
            start_time: Some(Utc::now()),
            submit_time: Utc::now(),
        });
    }

    let partitions = vec![
        Partition {
            name: "gpu".to_string(),
            total_nodes: 10,
            total_cpus: 640,
            state: "UP".to_string(),
        },
        Partition {
            name: "standard".to_string(),
            total_nodes: 20,
            total_cpus: 1280,
            state: "UP".to_string(),
        },
    ];

    ClusterStatus {
        nodes,
        jobs,
        partitions,
        updated_at,
    }
}
