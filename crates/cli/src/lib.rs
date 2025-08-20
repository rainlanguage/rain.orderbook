use crate::commands::local_db::{DbImport, DecodeEvents, EventsToSql, FetchEvents};
use crate::commands::{Chart, Order, Subgraph, Trade, Vault, Words};
use crate::execute::Execute;
use anyhow::Result;
use clap::Subcommand;
use rain_orderbook_quote::cli::Quoter;

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

    #[command(name = "fetch-events")]
    FetchEvents(FetchEvents),

    #[command(name = "decode-events")]
    DecodeEvents(DecodeEvents),

    #[command(name = "events-to-sql")]
    EventsToSql(EventsToSql),

    #[command(name = "db-import")]
    DbImport(DbImport),
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
            Orderbook::FetchEvents(fetch_events) => fetch_events.execute().await,
            Orderbook::DecodeEvents(decode_events) => decode_events.execute().await,
            Orderbook::EventsToSql(events_to_sql) => events_to_sql.execute().await,
            Orderbook::DbImport(db_import) => db_import.execute().await,
        }
    }
}
