use crate::commands::{Chart, Order, OrderTake, Vault};
use crate::execute::Execute;
use anyhow::Result;
use clap::Subcommand;
use rain_orderbook_quote::cli::QuoterCLi;

mod commands;
mod execute;
mod output;
mod status;
mod subgraph;
mod transaction;

#[derive(Subcommand)]
pub enum Orderbook {
    #[command(subcommand)]
    Order(Order),

    #[command(subcommand)]
    Vault(Box<Vault>),

    #[command(subcommand)]
    OrderTake(OrderTake),

    Chart(Chart),

    Quote(QuoterCLi),
}

impl Orderbook {
    pub async fn execute(self) -> Result<()> {
        match self {
            Orderbook::Order(order) => order.execute().await,
            Orderbook::Vault(vault) => (*vault).execute().await,
            Orderbook::OrderTake(order_take) => (order_take).execute().await,
            Orderbook::Chart(chart) => chart.execute().await,
            Orderbook::Quote(quote) => quote.execute().await,
        }
    }
}
