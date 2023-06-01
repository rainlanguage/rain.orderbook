use clap::{Subcommand};

#[derive(Subcommand)]
#[command(about = "Interact with an order(s) onchain and offchain.")]
pub enum Order {
    #[command(about = "List all orders from the subgraph.")]
    Ls
}

pub async fn ls() -> anyhow::Result<()> {
    crate::subgraph::orders::query().await?;
    Ok(())
}