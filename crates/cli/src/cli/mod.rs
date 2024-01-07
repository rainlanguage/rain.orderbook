use anyhow::Result;
use clap::command;
use clap::{Parser, Subcommand};
use crate::cli::order::Order;

mod order;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    orderbook: Orderbook,
}

#[derive(Subcommand)]
pub enum Orderbook {
    #[command(subcommand)]
    Order(Order)
}

pub async fn dispatch(orderbook: Orderbook) -> Result<()> {
    match orderbook {
        Orderbook::Order(order) => {
            match order {
                Order::Ls => Ok(order::ls().await?),
            }
        }
    }
}

pub async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;

    let cli = Cli::parse();
    dispatch(cli.orderbook).await
}
