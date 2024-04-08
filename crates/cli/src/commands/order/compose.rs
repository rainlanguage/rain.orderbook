use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Args, Clone)]
pub struct Compose {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    // path to the settings yaml
    #[arg(short = 'c', long, help = "Path to the settings yaml file")]
    settings_file: Option<PathBuf>,

    // the name of the scenrio to use
    #[arg(short = 's', long, help = "The name of the scenario to use")]
    scenario: String,

    // supported encoding
    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for Compose {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };
        let rainlang = DotrainOrder::new(dotrain, settings)
            .await?
            .compose_scenario_to_rainlang(self.scenario.clone())
            .await?;

        output(&None, self.encoding.clone(), rainlang.as_bytes())?;

        Ok(())
    }
}
