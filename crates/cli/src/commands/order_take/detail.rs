use crate::{execute::Execute, subgraph::CliSubgraphArgs};
use anyhow::Result;
use clap::Args;

use rain_orderbook_common::subgraph::SubgraphArgs;

use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderTakeDetailArgs {
    #[arg(short = 'i', long, help = "ID of the Order Take")]
    id: String,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderTakeDetailArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let order_take = subgraph_args
            .to_subgraph_client()
            .await?
            .order_take_detail(self.id.clone().into())
            .await?;
        info!("{:#?}", order_take);

        Ok(())
    }
}
