mod detail;
mod list;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use detail::CliOrderTradeDetailArgs;
use list::CliOrderTradesListArgs;

#[derive(Parser)]
pub enum Trade {
    #[command(about = "View an Order Take", alias = "view")]
    Detail(CliOrderTradeDetailArgs),

    #[command(about = "List takes for an Order", alias = "ls")]
    List(CliOrderTradesListArgs),
}

impl Execute for Trade {
    async fn execute(&self) -> Result<()> {
        match self {
            Trade::Detail(detail) => detail.execute().await,
            Trade::List(list) => list.execute().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_command() {
        Trade::command().debug_assert();
    }
}
