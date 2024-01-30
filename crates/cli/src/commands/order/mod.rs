mod add;
mod detail;
mod list;

use crate::execute::Execute;
use add::AddOrder;
use anyhow::Result;
use clap::Parser;
use detail::Detail;
use list::List;

#[derive(Parser)]
pub enum Order {
    #[command(about = "List all Orders", alias = "ls")]
    List(List),

    #[command(about = "View an Order", alias = "view")]
    Detail(Detail),

    #[command(about = "Create an Order", alias = "add")]
    Create(AddOrder),
}

impl Execute for Order {
    async fn execute(&self) -> Result<()> {
        match self {
            Order::List(list) => list.execute().await,
            Order::Detail(detail) => detail.execute().await,
            Order::Create(create) => create.execute().await,
        }
    }
}
