use crate::{
    execute::Execute, status::display_write_transaction_status, transaction::CliTransactionArgs,
};
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_common::add_order::AddOrderArgs;
use rain_orderbook_common::transaction::TransactionArgs;
use std::fs::read_to_string;
use std::path::PathBuf;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderAddArgs {
    #[arg(
        short = 'p',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_path: PathBuf,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,
}

impl TryFrom<CliOrderAddArgs> for AddOrderArgs {
    type Error = anyhow::Error;

    fn try_from(val: CliOrderAddArgs) -> Result<Self> {
        let text = read_to_string(val.dotrain_path).map_err(|e| anyhow!(e))?;
        Ok(Self { dotrain: text })
    }
}

impl Execute for CliOrderAddArgs {
    async fn execute(&self) -> Result<()> {
        let add_order_args: AddOrderArgs = self.clone().try_into()?;
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
