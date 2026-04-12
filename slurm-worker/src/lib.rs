use anyhow::Result;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use slurm_common::{RawSlurmInfo, SlurmClusterDiff};

#[derive(Deserialize, Serialize, Debug)]
pub struct PollConfig {
    pub interval: Duration,
}

pub struct PollLoop {
    interval: Duration,
    last_poll: Option<std::time::Instant>,
    previous_state: Option<RawSlurmInfo>,
    new_state: Option<RawSlurmInfo>,
}

impl PollLoop {
    pub async fn new(config: PollConfig) -> Result<PollLoop> {
        // Get the first cluster state
        Ok(PollLoop {
            interval: config.interval,
            last_poll: None,
            previous_state: None,
            new_state: None,
        })
    }

    pub async fn next<'src>(&'src mut self) -> Result<SlurmClusterDiff<'src>> {
        // wait until the next poll instance
        if let Some(last_poll) = self.last_poll {
            let next_time = last_poll + self.interval.to_std().unwrap();
            if std::time::Instant::now() < next_time {
                tokio::time::sleep_until(next_time.into()).await;
                self.last_poll = Some(std::time::Instant::now());
            }
        }
        if let Some(mut new_state) = self.new_state.take() {
            std::mem::swap(&mut self.previous_state, &mut new_state);
        }
        self.new_state = Some(RawSlurmInfo::from_local_system().await?);
        let old_state = self.previous_state.parse()?;
        let new_state = self.new_state.as_ref().map(|s| s.parse()).unwrap()?;
        let diff = old_state.diff(&new_state);
        Ok(diff)
    }
}
