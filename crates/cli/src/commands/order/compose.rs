use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::{Args, Parser};
use rain_orderbook_common::dotrain::Rebind;
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::collections::HashMap;
use std::error::Error;
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

    #[arg(short, long, value_parser= parse_key_val)]
    bind: Option<Vec<[String; 2]>>,

    // supported encoding
    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

/// Parse a single key-value pair
fn parse_key_val(
    key_value_pair: &str,
) -> Result<[String; 2], Box<dyn Error + Send + Sync + 'static>> {
    let pos = key_value_pair
        .find('=')
        .ok_or_else(|| format!("invalid key=value: no `=` found in `{key_value_pair}`"))?;
    Ok([
        key_value_pair[..pos].to_owned(),
        key_value_pair[pos + 1..].to_owned(),
    ])
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
        let binds: Vec<[String; 2]> = match &self.bind {
            Some(binds) => binds.clone(),
            None => vec![],
        };
        let rainlang = DotrainOrder::new(dotrain, settings)
            .await?
            .compose_scenario_to_rainlang(self.scenario.clone(), binds)
            .await?;

        output(&None, self.encoding.clone(), rainlang.as_bytes())?;

        Ok(())
    }
}
