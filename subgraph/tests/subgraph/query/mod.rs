pub(crate) mod orderbook;

use anyhow::Result;
use ethers::types::Address;
use orderbook::{get_orderbook_query, OrderBookResponse};

pub struct Query;

impl Query {
    pub async fn orderbook(address: Address) -> Result<OrderBookResponse> {
        get_orderbook_query(address).await
    }
}
