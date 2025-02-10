use cynic::Id;
use rain_orderbook_bindings::wasm_traits::prelude::*;
use rain_orderbook_subgraph_client::{OrderbookSubgraphClient, OrderbookSubgraphClientError};
use reqwest::Url;

/// Internal function to fetch a single transaction
/// Returns the Transaction struct
#[wasm_bindgen(js_name = "getTransaction")]
pub async fn get_sg_transaction(
    url: &str,
    id: &str,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let transaction = client.transaction_detail(Id::new(id)).await?;
    Ok(to_value(&transaction)?)
}
