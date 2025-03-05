use cynic::Id;
use rain_orderbook_subgraph_client::{OrderbookSubgraphClient, OrderbookSubgraphClientError};
use reqwest::Url;
use wasm_bindgen_utils::prelude::*;

/// Internal function to fetch Add Orders for a given transaction
/// Returns an array of AddOrder structs
#[wasm_bindgen(js_name = "getTransactionAddOrders")]
pub async fn get_transaction_add_orders(
    url: &str,
    tx_hash: &str,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let add_orders = client.transaction_add_orders(Id::new(tx_hash)).await?;
    Ok(to_js_value(&add_orders)?)
}
