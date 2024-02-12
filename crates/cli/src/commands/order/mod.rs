mod add;
mod detail;
mod list;
mod list_takes;
mod remove;

use crate::execute::Execute;
use add::CliOrderAddArgs;
use anyhow::Result;
use clap::Parser;
use detail::CliOrderDetailArgs;
use list::CliOrderListArgs;
use list_takes::CliOrderListTakes;
use remove::CliOrderRemoveArgs;

#[derive(Parser)]
pub enum Order {
    #[command(about = "List all Orders", alias = "ls")]
    List(CliOrderListArgs),

    #[command(about = "View an Order", alias = "view")]
    Detail(CliOrderDetailArgs),

    #[command(about = "Create an Order", alias = "add")]
    Create(CliOrderAddArgs),

    #[command(about = "Remove an Order", alias = "rm")]
    Remove(CliOrderRemoveArgs),

    #[command(about = "List takes for an Order", alias = "ls-takes")]
    ListTakes(CliOrderListTakes),
}

impl Execute for Order {
    async fn execute(&self) -> Result<()> {
        match self {
            Order::List(list) => list.execute().await,
            Order::Detail(detail) => detail.execute().await,
            Order::Create(create) => create.execute().await,
            Order::Remove(remove) => remove.execute().await,
            Order::ListTakes(list_takes) => list_takes.execute().await,
        }
    }
}
