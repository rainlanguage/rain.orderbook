mod list;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use list::CliOrderClearListArgs;

#[derive(Parser)]
pub enum OrderClear {
    #[command(about = "List all Order Clears", alias = "ls")]
    List(CliOrderClearListArgs),
}

impl Execute for OrderClear {
    async fn execute(&self) -> Result<()> {
        match self {
            OrderClear::List(list) => list.execute().await,
        }
    }
}
