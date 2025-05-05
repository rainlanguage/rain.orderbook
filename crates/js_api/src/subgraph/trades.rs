use super::SubgraphError;
use cynic::Id;
use rain_orderbook_subgraph_client::{
    types::common::SgTrade, OrderbookSubgraphClient, SgPaginationArgs,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetOrderTradesListResult(#[tsify(type = "SgTrade[]")] Vec<SgTrade>);
impl_wasm_traits!(GetOrderTradesListResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetOrderTradesCountResult(#[tsify(type = "number")] u64);
impl_wasm_traits!(GetOrderTradesCountResult);

/// Fetch trades for a specific order
/// Returns a list of Trade structs
#[wasm_export(
    js_name = "getOrderTradesList",
    unchecked_return_type = "GetOrderTradesListResult"
)]
pub async fn get_order_trades_list(
    url: &str,
    order_id: &str,
    pagination_args: SgPaginationArgs,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<GetOrderTradesListResult, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let trades = client
        .order_trades_list(
            Id::new(order_id),
            pagination_args,
            start_timestamp,
            end_timestamp,
        )
        .await?;
    Ok(GetOrderTradesListResult(trades))
}

/// Get details for a specific trade
/// Returns a Trade struct
#[wasm_export(js_name = "getOrderTradeDetail", unchecked_return_type = "SgTrade")]
pub async fn get_order_trade_detail(url: &str, trade_id: &str) -> Result<SgTrade, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let trade = client.order_trade_detail(Id::new(trade_id)).await?;
    Ok(trade)
}

/// Fetch the count of trades for a specific order
/// Returns the count as a JavaScript-compatible number
#[wasm_export(
    js_name = "getOrderTradesCount",
    unchecked_return_type = "GetOrderTradesCountResult"
)]
pub async fn get_order_trades_count(
    url: &str,
    order_id: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<GetOrderTradesCountResult, SubgraphError> {
    // Create the subgraph client using the provided URL
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);

    // Fetch all trades for the specific order and calculate the count
    let trades_count = client
        .order_trades_list_all(Id::new(order_id), start_timestamp, end_timestamp)
        .await?
        .len();

    // Convert the count to a JavaScript-compatible value and return
    Ok(GetOrderTradesCountResult(trades_count as u64))
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;

        #[tokio::test]
        async fn test_get_order_trades_list() {}

        #[tokio::test]
        async fn test_get_order_trade_detail() {}

        #[tokio::test]
        async fn test_get_order_trades_count() {}
    }
}
