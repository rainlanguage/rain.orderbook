use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_app_settings::{config_source::ConfigSource, Config};
use rain_orderbook_common::dotrain::RainDocument;
use rain_orderbook_common::rainlang::compose_to_rainlang;
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

    // the name of the scenrio to use
    #[arg(short = 's', long, help = "The name of the scenario to use")]
    scenario: String,

    // supported encoding
    #[arg(short = 'o', long, help = "Output encoding", default_value = "Binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for Compose {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let config_string = ConfigSource::try_from_string(frontmatter.to_string()).await?;
        let config: Config = config_string.try_into()?;

        let scenario = config.scenarios.get(&self.scenario).ok_or_else(|| {
            anyhow!(
                "Scenario {} not found in file {}",
                self.scenario,
                self.dotrain_file.display()
            )
        })?;

        let rainlang = compose_to_rainlang(dotrain, scenario.bindings.clone())?;

        output(&None, self.encoding.clone(), rainlang.as_bytes())?;

        Ok(())
    }
}
