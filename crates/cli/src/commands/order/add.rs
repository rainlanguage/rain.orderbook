use crate::{
    execute::Execute, status::display_write_transaction_status, transaction::CliTransactionArgs,
};
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_app_settings::Config;
use rain_orderbook_common::add_order::AddOrderArgs;
use rain_orderbook_common::frontmatter::parse_frontmatter;
use rain_orderbook_common::transaction::TransactionArgs;
use std::fs::read_to_string;
use std::ops::Deref;
use std::path::PathBuf;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderAddArgs {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short, long, help = "Deployment key to select from frontmatter")]
    deployment: String,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,
}

impl CliOrderAddArgs {
    async fn to_add_order_args(&self) -> Result<AddOrderArgs> {
        let text = read_to_string(&self.dotrain_file).map_err(|e| anyhow!(e))?;
        let config: Config = parse_frontmatter(text.clone())?.try_into()?;
        let config_deployment = config
            .deployments
            .get(&self.deployment)
            .ok_or(anyhow!("specified deployment is undefined!"))?;

        Ok(
            AddOrderArgs::new_from_deployment(text.clone(), config_deployment.deref().clone())
                .await?,
        )
    }
}

impl Execute for CliOrderAddArgs {
    async fn execute(&self) -> Result<()> {
        let add_order_args: AddOrderArgs = self.clone().to_add_order_args().await?;
        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;

        info!("----- Add Order -----");
        add_order_args
            .execute(tx_args, |status| {
                display_write_transaction_status(status);
            })
            .await?;

        Ok(())
    }
}
