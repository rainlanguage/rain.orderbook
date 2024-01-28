use crate::{
    execute::Execute, status::display_write_transaction_status,
    transaction::CliTransactionCommandArgs,
};
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_common::add_order::RemoveOrderArgs;
use rain_orderbook_common::transaction::TransactionArgs;
use std::fs::read_to_string;
use std::path::PathBuf;

pub type RemoveOrder = CliTransactionSubgraphCommandArgs<CliRemoveOrderArgs>;

impl Execute for RemoveOrder {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let order = subgraph_args
            .to_subgraph_client()
            .await?
            .order(self.cmd_args.order_id.clone().into())
            .await?
            .into();
        let remove_order_args: RemoveOrderArgs = order.try_into()?;

        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;

        println!("----- Remove Order -----");
        remove_order_args
            .execute(tx_args, |status| {
                display_write_transaction_status(status);
            })
            .await?;

        Ok(())
    }
}

#[derive(Args, Clone)]
pub struct CliRemoveOrderArgs {
    #[arg(
        short
        long,
        help = "ID of the Order"
    )]
    order_id: String,
}
