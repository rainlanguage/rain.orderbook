use crate::call::CliExecutableCommand;
use crate::commands::{Deposit, Order, Withdraw};
use anyhow::Result;
use clap::Subcommand;

mod call;
mod commands;
mod transaction;

#[derive(Subcommand)]
pub enum Orderbook {
    #[command(subcommand)]
    Order(Order),
    Deposit(Deposit),
    Withdraw(Withdraw),
}

impl Orderbook {
    pub async fn execute(self) -> Result<()> {
        match self {
            Orderbook::Order(order) => order.execute().await,
            Orderbook::Deposit(deposit) => deposit.execute().await,
            Orderbook::Withdraw(withdraw) => withdraw.execute().await,
        }
    }
}
