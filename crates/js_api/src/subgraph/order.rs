use cynic::Id;
use rain_orderbook_bindings::wasm_traits::prelude::*;
use rain_orderbook_subgraph_client::{
    types::common::OrdersListFilterArgs, MultiOrderbookSubgraphClient, MultiSubgraphArgs,
    OrderbookSubgraphClient, OrderbookSubgraphClientError, PaginationArgs,
};
use reqwest::Url;

/// Fetch all orders from multiple subgraphs
/// Returns a list of OrderWithSubgraphName structs
#[wasm_bindgen(js_name = "getOrders")]
pub async fn get_orders(
    subgraphs: Vec<MultiSubgraphArgs>,
    filter_args: OrdersListFilterArgs,
    pagination_args: PaginationArgs,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = MultiOrderbookSubgraphClient::new(subgraphs);
    let orders = client.orders_list(filter_args, pagination_args).await?;
    Ok(to_value(&orders)?)
}

/// Fetch a single order
/// Returns the Order struct
#[wasm_bindgen(js_name = "getOrder")]
pub async fn get_order(url: &str, id: &str) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let order = client.order_detail(Id::new(id)).await?;
    Ok(to_value(&order)?)
}

/// Get the volume information for an order's vaults
/// Returns a list of VaultVolume structs
#[wasm_bindgen(js_name = "getOrderVaultsVolume")]
pub async fn get_order_vaults_volume(
    url: &str,
    order_id: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let volumes = client
        .order_vaults_volume(Id::new(order_id), start_timestamp, end_timestamp)
        .await?;
    Ok(to_value(&volumes)?)
}

/// Get the list of trades for an order
/// Returns a list of Trade structs
#[wasm_bindgen(js_name = "getOrderTradesList")]
pub async fn get_order_trades_list(
    url: &str,
    order_id: &str,
    pagination_args: PaginationArgs,
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
    Ok(to_value(&trades)?)
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
    Ok(to_value(&trade)?)
}
