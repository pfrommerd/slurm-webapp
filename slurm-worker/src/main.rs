use anyhow::Result;
use clap::Parser;

use env_logger::Env;
use log::info;
use slurm_common::SlurmClusterDiff as ClusterDiff;
use std::path::PathBuf;

use slurm_common::ssh::{CargoTarget, SshOptions, Worker};
use slurm_worker::PollConfig;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = ".")]
    cargo_build_dir: String,
    #[arg(short, long, default_value = "10")]
    interval: u64,
    #[arg(short, long, default_value = "false")]
    /// Whether to run the worker in mock mode (just generates data, no actual worker child)
    mock: bool,
    #[clap(flatten)]
    ssh_options: SshOptions,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(
        Env::default().default_filter_or("info,slurm_worker=debug,slurm_common=debug"),
    )
    .init();

    let args = Args::parse();

    let ssh_config = args.ssh_options.resolve()?;
    let worker = CargoTarget {
        package: "slurm-worker".to_string(),
        binary: "slurm-worker-child".to_string(),
        cwd: PathBuf::from(args.cargo_build_dir),
    };
    let poll_config = PollConfig {
        interval: chrono::Duration::seconds(args.interval as i64),
    };
    let mut worker: Worker =
        Worker::launch(worker, vec![], ssh_config.as_ref(), Some(poll_config)).await?;
    while let Some::<ClusterDiff>(diff) = worker.next().await {
        info!("Diff: {:#?}", diff);
    }
    info!("Shutting down worker.");
    Ok(())
}
