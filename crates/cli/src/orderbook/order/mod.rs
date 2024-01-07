use clap::Subcommand;
use anyhow::Result;

#[derive(Subcommand)]
#[command(about = "Interact with an order(s) onchain and offchain.")]
pub enum Order {
    #[command(about = "List all orders from the subgraph.")]
    Ls,
}

pub async fn dispatch(order: Order) -> Result<()> {
    match order {
            Order::Ls => ls().await,
    }
}

pub async fn ls() -> anyhow::Result<()> {
    let orders = rain_orderbook_subgraph_queries::orders::query().await?;
    dbg!(orders);
    Ok(())
}