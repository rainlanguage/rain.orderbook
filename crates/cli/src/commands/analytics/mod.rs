mod downtime;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;

use downtime::DowntimeArgs;

#[derive(Parser)]
pub enum Analytics {
    #[command(
        about = "Provide KPI metrics (min, max, avg, count, total) for the downtime of all trades betwen a given time period, filtering out delays below the threshold."
    )]
    Downtime(DowntimeArgs),
}

impl Execute for Analytics {
    async fn execute(&self) -> Result<()> {
        match self {
            Analytics::Downtime(downtime) => downtime.execute().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_command() {
        Analytics::command().debug_assert();
    }
}
