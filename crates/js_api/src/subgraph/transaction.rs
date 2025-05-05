use super::SubgraphError;
use cynic::Id;
use rain_orderbook_subgraph_client::{types::common::SgTransaction, OrderbookSubgraphClient};
use reqwest::Url;
use wasm_bindgen_utils::prelude::*;

/// Internal function to fetch a single transaction
/// Returns the Transaction struct
#[wasm_export(js_name = "getTransaction", unchecked_return_type = "SgTransaction")]
pub async fn get_transaction(url: &str, tx_hash: &str) -> Result<SgTransaction, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    Ok(client.transaction_detail(Id::new(tx_hash)).await?)
}
