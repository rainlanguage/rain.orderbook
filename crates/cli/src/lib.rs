use crate::commands::local_db::{DbDump, SyncLocalDb};
use crate::commands::{Chart, Order, Subgraph, Trade, Vault, Words};
use crate::execute::Execute;
use anyhow::Result;
use clap::Subcommand;
use rain_orderbook_quote::cli::Quoter;

#[derive(Subcommand)]
#[command(about = "Local database operations")]
pub enum LocalDb {
    #[command(name = "dump")]
    Dump(DbDump),
    #[command(name = "sync")]
    Sync(SyncLocalDb),
}

impl LocalDb {
    pub async fn execute(self) -> Result<()> {
        match self {
            LocalDb::Dump(dump) => dump.execute().await,
            LocalDb::Sync(cmd) => cmd.execute().await,
        }
    }
}

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
    Vault(Vault),

    #[command(subcommand)]
    Trade(Trade),

    #[command(subcommand)]
    Subgraph(Subgraph),

    Chart(Chart),

    Quote(Quoter),

    Words(Words),

    #[command(name = "local-db", subcommand)]
    LocalDb(LocalDb),
}

impl Orderbook {
    pub async fn execute(self) -> Result<()> {
        match self {
            Orderbook::Order(order) => order.execute().await,
            Orderbook::Vault(vault) => vault.execute().await,
            Orderbook::Trade(trade) => trade.execute().await,
            Orderbook::Chart(chart) => chart.execute().await,
            Orderbook::Quote(quote) => quote.execute().await,
            Orderbook::Subgraph(subgraph) => subgraph.execute().await,
            Orderbook::Words(words) => words.execute().await,
            Orderbook::LocalDb(local_db) => local_db.execute().await,
        }
    }
}
