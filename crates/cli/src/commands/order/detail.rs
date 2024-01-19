use crate::{execute::Execute, subgraph::CliSubgraphCommandArgs};
use anyhow::Result;
use clap::Args;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderDetailArgs {
    #[arg(short, long, help = "ID of the Order")]
    order_id: String,
}

pub type Detail = CliSubgraphCommandArgs<CliOrderDetailArgs>;

impl Execute for Detail {
    async fn execute(&self) -> Result<()> {
        let order = self
            .subgraph_args
            .clone()
            .try_into_subgraph_client()
            .await?
            .order(self.cmd_args.order_id.clone().into())
            .await?;
        info!("{:#?}", order);

        Ok(())
    }
}
