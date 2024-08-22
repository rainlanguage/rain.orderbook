use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_app_settings::Config;
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Args, Clone)]
pub struct OrderbookAddress {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short = 'c', long, help = "Path to the settings yaml file")]
    settings_file: Option<PathBuf>,

    #[arg(short = 'e', long, help = "Deployment key to select from frontmatter")]
    deployment: String,

    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for OrderbookAddress {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };
        let order = DotrainOrder::new(dotrain, settings).await?;
        let order_config: Config = order.clone().config;

        let config_deployment = order_config
            .deployments
            .get(&self.deployment)
            .ok_or(anyhow!("specified deployment is undefined!"))?;

        let network_name = config_deployment.scenario.deployer.network.name.clone();

        let orderbook = order_config
            .orderbooks
            .get(&network_name)
            .ok_or(anyhow!("specified orderbook is undefined!"))?;

        let orderbook_address = orderbook.address.to_vec();
        output(&None, self.encoding.clone(), &orderbook_address)?;

        Ok(())
    }
}
