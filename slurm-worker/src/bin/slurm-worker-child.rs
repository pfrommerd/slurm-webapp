use anyhow::Result;
use slurm_worker::{PollConfig, PollLoop};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<()> {
    // read the config from stdin
    let stdin = BufReader::new(tokio::io::stdin());
    let config_json = stdin
        .lines()
        .next_line()
        .await?
        .expect("No config provided");
    let config = serde_json::from_str::<PollConfig>(&config_json)?;
    eprintln!("Worker process succesfully connected...");
    let mut poll_loop = match PollLoop::new(config).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to create poll loop: {}", e);
            return Err(anyhow::anyhow!("Failed to create poll loop"));
        }
    };
    loop {
        let diff = match poll_loop.next().await {
            Ok(diff) => diff,
            Err(e) => {
                eprintln!("Failed to get diff: {}", e);
                continue;
            }
        };
        println!("{}", serde_json::to_string(&diff).unwrap());
    }
}
