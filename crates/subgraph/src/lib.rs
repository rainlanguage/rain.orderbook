mod client;
mod cynic_client;
pub mod types;

#[cynic::schema("orderbook")]
pub mod schema {}

pub use client::{OrderbookSubgraphClient, OrderbookSubgraphClientError};
