mod list;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use list::CliOrderTakesListArgs;

#[derive(Parser)]
pub enum OrderTake {
    #[command(about = "List takes for an Order", alias = "ls")]
    List(CliOrderTakesListArgs),
}

impl Execute for OrderTake {
    async fn execute(&self) -> Result<()> {
        match self {
            OrderTake::List(list) => list.execute().await,
        }
    }
}
