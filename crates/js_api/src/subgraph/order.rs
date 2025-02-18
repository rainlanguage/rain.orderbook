use cynic::Id;
use rain_orderbook_common::types::OrderDetailExtended;
use rain_orderbook_subgraph_client::{
    types::common::{SgOrder, SgOrdersListFilterArgs},
    MultiOrderbookSubgraphClient, MultiSubgraphArgs, OrderbookSubgraphClient,
    OrderbookSubgraphClientError, SgPaginationArgs,
};
use reqwest::Url;
use wasm_bindgen_utils::prelude::*;

/// Internal function to fetch a single order
/// Returns the SgOrder struct
pub async fn get_sg_order(url: &str, id: &str) -> Result<SgOrder, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let order = client.order_detail(Id::new(id)).await?;
    Ok(order)
}

/// Fetch all orders from multiple subgraphs
/// Returns a list of OrderWithSubgraphName structs
#[wasm_bindgen(js_name = "getOrders")]
pub async fn get_orders(
    subgraphs: Vec<MultiSubgraphArgs>,
    filter_args: SgOrdersListFilterArgs,
    pagination_args: SgPaginationArgs,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = MultiOrderbookSubgraphClient::new(subgraphs);
    let orders = client.orders_list(filter_args, pagination_args).await?;
    Ok(to_js_value(&orders)?)
}

/// Fetch a single order
/// Returns the SgOrder struct
#[wasm_bindgen(js_name = "getOrder")]
pub async fn get_order(url: &str, id: &str) -> Result<JsValue, OrderbookSubgraphClientError> {
    let order = get_sg_order(url, id).await?;
    Ok(to_js_value(&order)?)
}

/// Extend an order to include Rainlang string
/// Returns an OrderDetailExtended struct
#[wasm_bindgen(js_name = "extendOrder")]
pub fn order_detail_extended(order: SgOrder) -> Result<JsValue, OrderbookSubgraphClientError> {
    let order_extended: OrderDetailExtended = order
        .try_into()
        .map_err(|_| OrderbookSubgraphClientError::OrderDetailExtendError)?;
    Ok(to_js_value(&order_extended)?)
}

/// Fetch trades for a specific order
/// Returns a list of Trade structs
#[wasm_bindgen(js_name = "getOrderTradesList")]
pub async fn get_order_trades_list(
    url: &str,
    order_id: &str,
    pagination_args: SgPaginationArgs,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let trades = client
        .order_trades_list(
            Id::new(order_id),
            pagination_args,
            start_timestamp,
            end_timestamp,
        )
        .await?;
    Ok(to_js_value(&trades)?)
}

/// Get details for a specific trade
/// Returns a Trade struct
#[wasm_bindgen(js_name = "getOrderTradeDetail")]
pub async fn get_order_trade_detail(
    url: &str,
    trade_id: &str,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let trade = client.order_trade_detail(Id::new(trade_id)).await?;
    Ok(to_js_value(&trade)?)
}

/// Fetch the count of trades for a specific order
/// Returns the count as a JavaScript-compatible number
#[wasm_bindgen(js_name = "getOrderTradesCount")]
pub async fn get_order_trades_count(
    url: &str,
    order_id: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    // Create the subgraph client using the provided URL
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);

    // Fetch all trades for the specific order and calculate the count
    let trades_count = client
        .order_trades_list_all(Id::new(order_id), start_timestamp, end_timestamp)
        .await?
        .len();

    // Convert the count to a JavaScript-compatible value and return
    Ok(to_js_value(&trades_count)?)
}

/// Fetch volume information for vaults associated with an order
#[wasm_bindgen(js_name = "getOrderVaultsVolume")]
pub async fn order_vaults_volume(
    url: &str,
    order_id: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let volumes = client
        .order_vaults_volume(Id::new(order_id), start_timestamp, end_timestamp)
        .await?;
    Ok(to_js_value(&volumes)?)
}

/// Measures an order's performance (including vaults apy and vol and total apy and vol)
#[wasm_bindgen(js_name = "getOrderPerformance")]
pub async fn order_performance(
    url: &str,
    order_id: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let performance = client
        .order_performance(Id::new(order_id), start_timestamp, end_timestamp)
        .await?;
    Ok(to_js_value(&performance)?)
}
