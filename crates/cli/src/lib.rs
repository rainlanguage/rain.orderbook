use crate::commands::local_db::{
    DbDump, DecodeEvents, DecodedEventsToSql, FetchEvents, TokensFetch, TokensToSql,
};
use crate::commands::{Chart, Order, Subgraph, Trade, Vault, Words};
use crate::execute::Execute;
use anyhow::Result;
use clap::Subcommand;
use rain_orderbook_quote::cli::Quoter;

#[derive(Subcommand)]
#[command(about = "Local database operations")]
pub enum LocalDb {
    #[command(name = "fetch-events")]
    FetchEvents(FetchEvents),
    #[command(name = "decode-events")]
    DecodeEvents(DecodeEvents),
    #[command(name = "decoded-events-to-sql")]
    DecodedEventsToSql(DecodedEventsToSql),
    #[command(name = "dump")]
    Dump(DbDump),
    #[command(name = "tokens-fetch")]
    TokensFetch(TokensFetch),
    #[command(name = "tokens-to-sql")]
    TokensToSql(TokensToSql),
}

impl LocalDb {
    pub async fn execute(self) -> Result<()> {
        match self {
            LocalDb::FetchEvents(fetch_events) => fetch_events.execute().await,
            LocalDb::DecodeEvents(decode_events) => decode_events.execute().await,
            LocalDb::DecodedEventsToSql(decoded_events_to_sql) => {
                decoded_events_to_sql.execute().await
            }
            LocalDb::Dump(dump) => dump.execute().await,
            LocalDb::TokensFetch(cmd) => cmd.execute().await,
            LocalDb::TokensToSql(cmd) => cmd.execute().await,
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
