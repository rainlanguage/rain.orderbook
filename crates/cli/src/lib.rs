use anyhow::Result;
use clap::Subcommand;

use crate::commands::{Deposit, Order};

mod commands;
mod transaction;

#[derive(Subcommand)]
pub enum Orderbook {
    #[command(subcommand)]
    Order(Order),
    Deposit(Deposit),
}

impl Orderbook {
    pub async fn execute(self) -> Result<()> {
        match self {
            Orderbook::Order(order) => order.execute().await,
            Orderbook::Deposit(deposit) => deposit.execute().await,
        }
    }
}
