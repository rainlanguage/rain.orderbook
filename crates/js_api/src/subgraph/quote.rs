use rain_orderbook_bindings::wasm_traits::prelude::*;
use rain_orderbook_subgraph_client::{
    types::common::Order,
    OrderbookSubgraphClientError,
};
use alloy::primitives::U256;

/// Get quotes for batch orders
/// Returns a list of BatchOrderQuotesResponse structs
#[wasm_bindgen(js_name = "getOrderQuotes")]
pub async fn get_order_quotes(
    orders: Vec<Order>,
    block_number: Option<u64>,
    rpc_url: &str,
    gas: Option<String>,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let gas = gas
        .map(|g| U256::from_str(&g))
        .transpose()
        .map_err(|e| OrderbookSubgraphClientError::ParseError(e))?;

    let quotes = rain_orderbook_quote::get_order_quotes(
        orders,
        block_number,
        rpc_url.to_string(),
        gas
    ).await?;

    Ok(to_value(&quotes)?)
}
