mod list;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use list::List;

#[derive(Parser)]
pub enum Order {
    #[command(about = "List all orders from the subgraph.", alias = "ls")]
    List(List),
}

impl Execute for Order {
    async fn execute(&self) -> Result<()> {
        match self {
            Order::List(list) => list.execute().await,
        }
    }
}
