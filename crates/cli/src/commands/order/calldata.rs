use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;
use rain_orderbook_app_settings::{
    Config
};
use rain_orderbook_common::add_order::AddOrderArgs;
use std::ops::Deref;
use alloy::sol_types::SolCall;

#[derive(Args, Clone)]
pub struct Calldata {
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

    #[arg(short, long, help = "RPC URL")]
    pub rpc_url: String,

}

impl Execute for Calldata {
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
        let dotrain_string: String = order.clone().dotrain;

        let config_deployment = order_config
            .deployments
            .get(&self.deployment)
            .ok_or(anyhow!("specified deployment is undefined!"))?;

        let add_order_args = AddOrderArgs::new_from_deployment(dotrain_string, config_deployment.deref().clone())
                .await;

        let add_order_calldata = add_order_args?
            .try_into_call(self.rpc_url.clone())
            .await?
            .abi_encode();
        
        output(&None, self.encoding.clone(), &add_order_calldata)?;

        Ok(())
    }
}
