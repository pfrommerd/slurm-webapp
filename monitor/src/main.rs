use anyhow::{Context, Result};
use chrono::Utc;
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
            debug!("Received diff: {:#?}", diff);
            // Apply in-memory
            status.apply(diff.clone());

            // Apply to DB
            if let Err(e) = slurm_common::db::apply_diff(&pool, diff).await {
                error!("Error applying diff: {}", e);
            } else {
                if let Err(e) =
                    slurm_common::db::update_metadata(&pool, "last_updated", &Utc::now()).await
                {
                    error!("Error updating metadata: {}", e);
                }
                info!("Updated cluster status at {}", status.updated_at);
            }
        } else {
            error!("Failed to parse line as ClusterStatus: {}", line);
        }
    }

    Ok(())
}
