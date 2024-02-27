mod detail;
mod list;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use detail::CliOrderTakeDetailArgs;
use list::CliOrderTakesListArgs;

#[derive(Parser)]
pub enum OrderTake {
    #[command(about = "View an Order Take", alias = "view")]
    Detail(CliOrderTakeDetailArgs),

    #[command(about = "List takes for an Order", alias = "ls")]
    List(CliOrderTakesListArgs),
}

impl Execute for OrderTake {
    async fn execute(&self) -> Result<()> {
        match self {
            OrderTake::Detail(detail) => detail.execute().await,
            OrderTake::List(list) => list.execute().await,
        }
    }
}
