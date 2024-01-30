use crate::{execute::Execute, subgraph::CliSubgraphArgs};
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::subgraph::SubgraphArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderDetailArgs {
    #[arg(short='i', long, help = "ID of the Order")]
    order_id: String,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderDetailArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let order = subgraph_args
            .to_subgraph_client()
            .await?
            .order(self.order_id.clone().into())
            .await?;
        info!("{:#?}", order);

        Ok(())
    }
}
