use anyhow::Result;
use clap::Subcommand;

mod order;
mod deposit;

#[derive(Subcommand)]
pub enum Orderbook {
    #[command(subcommand)]
    Order(order::Order),
    #[command(about = "Deposit funds into a vault.")]
    Deposit(deposit::DepositArgs)
}

pub async fn dispatch(orderbook: Orderbook) -> Result<()> {
    match orderbook {
        Orderbook::Order(order) => order::dispatch(order).await,
        Orderbook::Deposit(args) => deposit::deposit(args).await,
    }
}