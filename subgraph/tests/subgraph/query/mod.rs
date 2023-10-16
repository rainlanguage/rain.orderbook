pub(crate) mod orderbook;
pub(crate) mod rain_meta_v1;

use anyhow::Result;
use ethers::types::{Address, Bytes};
use orderbook::{get_orderbook_query, OrderBookResponse};
use rain_meta_v1::{get_rain_meta_v1, RainMetaV1Response};

pub struct Query;

impl Query {
    pub async fn orderbook(address: Address) -> Result<OrderBookResponse> {
        get_orderbook_query(address).await
    }

    pub async fn rain_meta_v1(id: Bytes) -> Result<RainMetaV1Response> {
        get_rain_meta_v1(id).await
    }
}
