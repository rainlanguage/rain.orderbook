mod detail;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use detail::CliOrderTakeDetailArgs;

#[derive(Parser)]
pub enum OrderTake {
    #[command(about = "View an Order Take", alias = "view")]
    Detail(CliOrderTakeDetailArgs),
}

impl Execute for OrderTake {
    async fn execute(&self) -> Result<()> {
        match self {
            OrderTake::Detail(detail) => detail.execute().await,
        }
    }
}
