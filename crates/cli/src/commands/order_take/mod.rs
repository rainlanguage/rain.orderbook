mod detail;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use detail::CliOrderTakeDetailArgs;

#[derive(Parser)]
pub enum OrderTake {
    #[command(about = "List all Takes for an Order", alias = "ls")]
    Detail(CliOrderTakeDetailArgs),
}

impl Execute for OrderTake {
    async fn execute(&self) -> Result<()> {
        match self {
            OrderTake::Detail(detail) => detail.execute().await,
        }
    }
}
