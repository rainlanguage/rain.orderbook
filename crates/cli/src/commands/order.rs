use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub enum Order {
    #[command(about = "List all orders from the subgraph.")]
    Ls,
}

impl Order {
    pub async fn execute(self) -> Result<()> {
        match self {
            Order::Ls => ls().await,
        }
    }
}

pub async fn ls() -> anyhow::Result<()> {
    let orders = rain_orderbook_subgraph_queries::orders::query().await?;
    dbg!(orders);
    Ok(())
}
