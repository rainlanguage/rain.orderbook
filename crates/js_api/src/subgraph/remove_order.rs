use cynic::Id;
use rain_orderbook_subgraph_client::{OrderbookSubgraphClient, OrderbookSubgraphClientError};
use reqwest::Url;
use wasm_bindgen_utils::prelude::*;

/// Internal function to fetch Remove Orders for a given transaction
/// Returns an array of RemoveOrder structs
#[wasm_bindgen(js_name = "getTransactionRemoveOrders")]
pub async fn get_transaction_remove_orders(
    url: &str,
    tx_hash: &str,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let remove_orders = client.transaction_remove_orders(Id::new(tx_hash)).await?;
    Ok(to_js_value(&remove_orders)?)
}
