use crate::execute::Execute;
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_common::fuzz::{FuzzRunner, FuzzRunnerContext};
use std::fs::read_to_string;
use std::path::PathBuf;
use tracing::info;

#[derive(Args, Clone)]
pub struct Chart {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,
}

impl Execute for Chart {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let fuzzer = FuzzRunner::new(None);
        let mut context = FuzzRunnerContext::new(&dotrain, None, None)?;
        let chart_data = fuzzer.make_chart_data(&mut context).await?;

        info!("{:#?}", chart_data);
        Ok(())
    }
}
