mod list;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use list::List;

#[derive(Parser)]
pub enum Order {
    #[command(about = "List all orders from the subgraph.")]
    Ls(List),
}

impl Execute for Order {
    async fn execute(&self) -> Result<()> {
        match self {
            Order::Ls(list) => list.execute().await,
        }
    }
}
