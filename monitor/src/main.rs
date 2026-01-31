use anyhow::{Context, Result};
use clap::Parser;

use env_logger::Env;
use log::{debug, error, info, warn};
use serde::Deserialize;
use slurm_common::{ClusterDiff, ClusterState};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

mod ssh;
use ssh::{Process, SshOptions};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        long,
        default_value = "cargo build --bin worker --release --target x86_64-unknown-linux-musl"
    )]
    cargo_build_cmd: String,
    #[arg(long, default_value = "worker")]
    cargo_build_cwd: String,

    #[arg(long, default_value = "false")]
    /// Whether to locate the worker binary using "cargo build" instead of "which"
    cargo_build: bool,

    #[arg(short, long, default_value = "false")]
    /// Whether to run the worker in mock mode
    mock: bool,

    #[clap(flatten)]
    ssh_options: SshOptions,
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

    let worker_path = if args.cargo_build {
        build_worker(&args.cargo_build_cmd, &args.cargo_build_cwd).await?
    } else {
        PathBuf::from(which::which("worker")?.to_string_lossy().to_string())
    };
    info!("Using worker binary at: {:?}", worker_path);

    let mut proc = match launch_worker(&args, worker_path).await {
        Ok(proc) => proc,
        Err(e) => {
            error!("Failed to launch worker: {}", e);
            return Err(e);
        }
    };
    info!("Monitor started. Waiting for worker updates.");
    monitor_loop(&mut *proc, pool).await
}

async fn launch_worker(args: &Args, worker_path: PathBuf) -> Result<Box<dyn Process>> {
    if let Some(options) = &args.ssh_options.resolve()? {
        let mut remote_args = Vec::new();
        if args.mock {
            remote_args.push("--mock".to_string());
        }
        info!("Launching worker via SSH on {}", options.host);
        let child = ssh::launch_on_remote(worker_path, remote_args, options).await?;
        let proc: Box<dyn Process> = Box::new(child);
        Ok(proc)
    } else {
        let mut command = Command::new(worker_path);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        if args.mock {
            command.arg("--mock");
        }
        let child = command.spawn().context("Failed to spawn worker process")?;
        let proc: Box<dyn Process> = Box::new(child);
        Ok(proc)
    }
}

async fn monitor_loop(child: &mut dyn Process, pool: Pool<Sqlite>) -> Result<()> {
    // Parse the command string (simplistic splitting)
    let stdout = child.stdout().context("Failed to open stdout")?;
    let stderr = child.stderr().context("Failed to open stderr")?;

    // Read both the stdout and stderr asynchronously
    let mut stdout_reader = stdout.lines();
    let mut stderr_reader = stderr.lines();

    let mut status = ClusterState::default();

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if let Ok(diff) = serde_json::from_str::<ClusterDiff>(&line) {
                            debug!("Received diff: {:#?}", diff);
                            // Apply in-memory
                            status.apply(diff.clone());

                            // Apply to DB
                            if let Err(e) = slurm_common::db::apply_diff(&pool, diff).await {
                                error!("Error applying diff: {}", e);
                            } else {
                                info!("Updated cluster status.");
                            }
                        } else {
                            error!("Failed to parse line as ClusterStatus: {}", line);
                        }
                    }
                    Ok(None) => {
                        warn!("Worker process died.");
                        break;
                    },
                    Err(e) => {
                        error!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => error!("Worker stderr: {}", line),
                    Ok(None) => (), // stderr closed, ignore? Or maybe worker died?
                    Err(e) => error!("Error reading stderr: {}", e),
                }
            }
        }
    }
    Ok(())
}

#[derive(Deserialize, Debug)]
#[serde(tag = "reason")]
pub enum BuildOutput {
    #[serde(rename = "build-finished")]
    BuildFinished,
    #[serde(rename = "compiler-artifact")]
    CompilerArtifact {
        package_id: String,
        target: serde_json::Value,
        profile: serde_json::Value,
        filenames: Vec<String>,
        executable: Option<String>,
    },
    #[serde(other)]
    Other,
}

// Will return the path to the worker binary
async fn build_worker(cargo_cmd: &str, cwd: &str) -> Result<PathBuf> {
    let cmd_str = cargo_cmd.to_string() + " --message-format=json";
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Worker command cannot be empty");
    }
    let program = parts[0];
    let args = &parts[1..];

    let mut command = Command::new(program);
    command.args(args).stdout(Stdio::piped()).current_dir(cwd);

    info!("Executing cargo build: {}", cargo_cmd);

    let mut child = command
        .spawn()
        .context("Failed to spawn cargo build process")?;
    let stdout = child.stdout.take().context("Failed to open stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    let mut last_artifact = None;
    while let Some(line) = reader.next_line().await? {
        let output = serde_json::from_str::<BuildOutput>(&line)?;
        match output {
            BuildOutput::CompilerArtifact { .. } => last_artifact = Some(output),
            BuildOutput::BuildFinished => break,
            _ => (),
        }
    }
    if let Some(BuildOutput::CompilerArtifact { executable, .. }) = last_artifact {
        return Ok(PathBuf::from(executable.context("No executable found")?));
    } else {
        return Err(anyhow::anyhow!("No executable foundin build output"));
    }
}
