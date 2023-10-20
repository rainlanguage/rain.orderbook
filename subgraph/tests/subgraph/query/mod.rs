pub(crate) mod content_meta_v1;
pub(crate) mod order;
pub(crate) mod orderbook;
pub(crate) mod rain_meta_v1;

use anyhow::Result;
use ethers::types::{Address, Bytes};
use once_cell::sync::Lazy;
use reqwest::Url;

use content_meta_v1::{get_content_meta_v1, ContentMetaV1Response};
use order::{get_order, OrderResponse};
use orderbook::{get_orderbook_query, OrderBookResponse};
use rain_meta_v1::{get_rain_meta_v1, RainMetaV1Response};

pub static SG_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("http://localhost:8000/subgraphs/name/test/test").unwrap());

pub struct Query;

impl Query {
    pub async fn orderbook(address: Address) -> Result<OrderBookResponse> {
        get_orderbook_query(address).await
    }

    pub async fn rain_meta_v1(id: Bytes) -> Result<RainMetaV1Response> {
        get_rain_meta_v1(id).await
    }

    pub async fn content_meta_v1(id: Bytes) -> Result<ContentMetaV1Response> {
        get_content_meta_v1(id).await
    }

    pub async fn order(id: Bytes) -> Result<OrderResponse> {
        get_order(id).await
    }
}
