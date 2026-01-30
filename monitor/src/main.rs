use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{debug, error, info};
use slurm_common::{ClusterDiff, ClusterStatus};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "cargo run --bin worker -- --mock")]
    /// Command to run the worker (can be an ssh command)
    worker_cmd: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    let args = Args::parse();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await
        .context(
            "Failed to connect to database. Make sure to create the file first if using sqlite, or let the backend run migrations.",
        )?;

    info!(
        "Monitor started. Connecting to worker command: {}",
        args.worker_cmd
    );

    monitor_loop(args.worker_cmd, pool).await
}

async fn monitor_loop(cmd_str: String, pool: Pool<Sqlite>) -> Result<()> {
    // Parse the command string (simplistic splitting)
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Worker command cannot be empty");
    }
    let program = parts[0];
    let args = &parts[1..];

    let mut command = Command::new(program);
    command.args(args);
    command.stdout(Stdio::piped());

    let mut child = command.spawn().context("Failed to spawn worker process")?;
    let stdout = child.stdout.take().context("Failed to open stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    let mut status = ClusterStatus::default();

    while let Some(line) = reader.next_line().await? {
        if let Ok(diff) = serde_json::from_str::<ClusterDiff>(&line) {
            debug!("Received diff: {:#?}", diff);
            status.apply(diff);
            if let Err(e) = update_db(&pool, &status).await {
                error!("Error updating DB: {}", e);
            } else {
                info!("Updated cluster status at {}", status.updated_at);
            }
        } else {
            error!("Failed to parse line as ClusterStatus: {}", line);
        }
    }

    Ok(())
}

async fn update_db(pool: &Pool<Sqlite>, status: &ClusterStatus) -> Result<()> {
    // Upsert Nodes
    for node in &status.nodes {
        let resources = serde_json::to_string(&node.resources).unwrap_or_default();
        sqlx::query!(
            r#"
            INSERT INTO nodes (name, state, cpus, real_memory, resources, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET
                state = excluded.state,
                cpus = excluded.cpus,
                real_memory = excluded.real_memory,
                resources = excluded.resources,
                updated_at = excluded.updated_at
            "#,
            node.name,
            node.state,
            node.cpus,
            node.real_memory,
            resources,
            status.updated_at
        )
        .execute(pool)
        .await?;
    }

    // Upsert Jobs
    for job in &status.jobs {
        let state = job.state.as_ref();
        sqlx::query!(
            r#"
            INSERT INTO jobs (job_id, user, partition, state, num_nodes, num_cpus, time_limit, start_time, submit_time, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(job_id) DO UPDATE SET
                state = excluded.state,
                updated_at = excluded.updated_at
            "#,
            job.job_id,
            job.user,
            job.partition,
            state,
            job.num_nodes,
            job.num_cpus,
            job.time_limit,
            job.start_time,
            job.submit_time,
            status.updated_at
        )
        .execute(pool)
        .await?;
    }

    // Upsert Partitions
    for part in &status.partitions {
        sqlx::query!(
            r#"
            INSERT INTO partitions (name, total_nodes, total_cpus, state, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET
                total_nodes = excluded.total_nodes,
                total_cpus = excluded.total_cpus,
                state = excluded.state,
                updated_at = excluded.updated_at
            "#,
            part.name,
            part.total_nodes,
            part.total_cpus,
            part.state,
            status.updated_at
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
