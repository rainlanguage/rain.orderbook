use crate::commands::{Deposit, Order, Withdraw, AddOrder};
use crate::execute::Execute;
use anyhow::Result;
use clap::Subcommand;

mod commands;
mod execute;
mod transaction;

#[derive(Subcommand)]
pub enum Orderbook {
    #[command(subcommand)]
    Order(Order),
    Deposit(Deposit),
    Withdraw(Withdraw),
    AddOrder(AddOrder),
}

impl Orderbook {
    pub async fn execute(self) -> Result<()> {
        match self {
            Orderbook::Order(order) => order.execute().await,
            Orderbook::Deposit(deposit) => deposit.execute().await,
            Orderbook::Withdraw(withdraw) => withdraw.execute().await,
            Orderbook::AddOrder(add_order) => add_order.execute().await,
        }
    }
}
