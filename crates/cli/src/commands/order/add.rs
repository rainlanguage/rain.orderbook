use crate::{
    execute::Execute, status::display_write_transaction_status,
    transaction::CliTransactionCommandArgs,
};
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_common::add_order::AddOrderArgs;
use rain_orderbook_common::transaction::TransactionArgs;
use std::fs::read_to_string;
use std::path::PathBuf;

pub type AddOrder = CliTransactionCommandArgs<CliAddOrderArgs>;

impl Execute for AddOrder {
    async fn execute(&self) -> Result<()> {
        let add_order_args: AddOrderArgs = self.cmd_args.clone().try_into()?;
        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;

        println!("----- Add Order -----");
        add_order_args
            .execute(tx_args, |status| {
                display_write_transaction_status(status);
            })
            .await?;

        Ok(())
    }
}

#[derive(Args, Clone)]
pub struct CliAddOrderArgs {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,
}

impl TryFrom<CliAddOrderArgs> for AddOrderArgs {
    type Error = anyhow::Error;

    fn try_from(val: CliAddOrderArgs) -> Result<Self> {
        let text = read_to_string(val.dotrain_file).map_err(|e| anyhow!(e))?;
        Ok(Self { dotrain: text })
    }
}
