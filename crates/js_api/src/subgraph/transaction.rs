use cynic::Id;
use rain_orderbook_subgraph_client::{OrderbookSubgraphClient, OrderbookSubgraphClientError};
use reqwest::Url;
use wasm_bindgen_utils::prelude::*;

/// Internal function to fetch a single transaction
/// Returns the Transaction struct
#[wasm_bindgen(js_name = "getTransaction")]
pub async fn get_transaction(
    url: &str,
    tx_hash: &str,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let transaction = client.transaction_detail(Id::new(tx_hash)).await?;
    Ok(to_js_value(&transaction)?)
}
