mod add;
mod calldata;
mod compose;
mod detail;
mod list;
mod remove;

use crate::execute::Execute;
use add::CliOrderAddArgs;
use anyhow::Result;
use calldata::AddOrderCalldata;
use clap::Parser;
use compose::Compose;

use detail::CliOrderDetailArgs;
use list::CliOrderListArgs;
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

    #[command(about = "Compose a .rain order file to Rainlang", alias = "comp")]
    Compose(Compose),

    #[command(
        about = "Generate calldata for addOrder from a composition",
        alias = "call"
    )]
    Calldata(AddOrderCalldata),
}

impl Execute for Order {
    async fn execute(&self) -> Result<()> {
        match self {
            Order::List(list) => list.execute().await,
            Order::Detail(detail) => detail.execute().await,
            Order::Create(create) => create.execute().await,
            Order::Remove(remove) => remove.execute().await,
            Order::Compose(compose) => compose.execute().await,
            Order::Calldata(calldata) => calldata.execute().await,
        }
    }
}
