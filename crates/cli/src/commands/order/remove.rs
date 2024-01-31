use crate::{
    execute::Execute, status::display_write_transaction_status,
    transaction::CliTransactionSubgraphCommandArgs,
};
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::remove_order::RemoveOrderArgs;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_common::transaction::TransactionArgs;

pub type RemoveOrder = CliTransactionSubgraphCommandArgs<CliRemoveOrderArgs>;

impl Execute for RemoveOrder {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let order = subgraph_args
            .to_subgraph_client()
            .await?
            .order(self.cmd_args.order_id.clone().into())
            .await?;
        let remove_order_args: RemoveOrderArgs = order.into();

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
    #[arg(short = 'i', long, help = "ID of the Order")]
    order_id: String,
}
