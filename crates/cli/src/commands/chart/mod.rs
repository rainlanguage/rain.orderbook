use crate::execute::Execute;
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_app_settings::{config_source::ConfigSource, Config};
use rain_orderbook_common::dotrain::RainDocument;
use rain_orderbook_common::fuzz::FuzzRunner;
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
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let config_string = ConfigSource::try_from_string(frontmatter.to_string()).await?;
        let config: Config = config_string.try_into()?;
        let fuzzer = FuzzRunner::new(&dotrain, config, None).await;
        let chart_data = fuzzer.make_chart_data().await?;

        info!("{:#?}", chart_data);
        Ok(())
    }
}
