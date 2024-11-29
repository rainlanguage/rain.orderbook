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
