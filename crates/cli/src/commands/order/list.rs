use crate::execute::Execute;
use anyhow::Result;
use clap::Args;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderListArgs {}

pub type List = CliOrderListArgs;

impl Execute for List {
    async fn execute(&self) -> Result<()> {
        let orders = rain_orderbook_subgraph_queries::orders::query().await?;
        info!("{:?}", orders);

        Ok(())
    }
}
