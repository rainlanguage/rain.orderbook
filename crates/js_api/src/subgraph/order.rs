use std::collections::{HashMap, HashSet};

use cynic::Id;
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};
use rain_orderbook_common::types::OrderDetailExtended;
use rain_orderbook_subgraph_client::{
    types::common::{Order, OrdersListFilterArgs, Vault},
    MultiOrderbookSubgraphClient, MultiSubgraphArgs, OrderbookSubgraphClient,
    OrderbookSubgraphClientError, PaginationArgs,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Tsify)]
pub struct OrderWithSortedVaults {
    #[tsify(type = "OrderSubgraph")]
    pub order: Order,
    pub vaults: HashMap<String, Vec<Vault>>,
}
impl_all_wasm_traits!(OrderWithSortedVaults);

/// Internal function to fetch a single order
/// Returns the Order struct
pub async fn get_sg_order(url: &str, id: &str) -> Result<Order, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let order = client.order_detail(Id::new(id)).await?;
    Ok(order)
}

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

fn sort_vaults(order: &Order) -> HashMap<String, Vec<Vault>> {
    let mut sorted_vaults: HashMap<String, Vec<Vault>> = HashMap::new();

    let input_ids: HashSet<_> = order.inputs.iter().map(|v| &v.vault_id).collect();
    let output_ids: HashSet<_> = order.outputs.iter().map(|v| &v.vault_id).collect();

    sorted_vaults.insert("inputs".to_string(), Vec::new());
    sorted_vaults.insert("outputs".to_string(), Vec::new());
    sorted_vaults.insert("inputs_outputs".to_string(), Vec::new());

    for vault in &order.inputs {
        if output_ids.contains(&vault.vault_id) {
            sorted_vaults
                .get_mut("inputs_outputs")
                .unwrap()
                .push(vault.clone());
        } else {
            sorted_vaults.get_mut("inputs").unwrap().push(vault.clone());
        }
    }

    for vault in &order.outputs {
        if !input_ids.contains(&vault.vault_id) {
            sorted_vaults
                .get_mut("outputs")
                .unwrap()
                .push(vault.clone());
        }
    }

    sorted_vaults
}

/// Fetch a single order
/// Returns the Order struct with sorted vaults
#[wasm_bindgen(js_name = "getOrder")]
pub async fn get_order(url: &str, id: &str) -> Result<JsValue, OrderbookSubgraphClientError> {
    let order = get_sg_order(url, id).await?;
    Ok(to_value(&OrderWithSortedVaults {
        order: order.clone(),
        vaults: sort_vaults(&order),
    })?)
}

/// Extend an order to include Rainlang string
/// Returns an OrderDetailExtended struct
#[wasm_bindgen(js_name = "extendOrder")]
pub fn order_detail_extended(order: Order) -> Result<JsValue, OrderbookSubgraphClientError> {
    let order_extended: OrderDetailExtended = order
        .try_into()
        .map_err(|_| OrderbookSubgraphClientError::OrderDetailExtendError)?;
    Ok(to_value(&order_extended)?)
}

/// Fetch trades for a specific order
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
    Ok(to_value(&trades_count)?)
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
    Ok(to_value(&volumes)?)
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
    Ok(to_value(&performance)?)
}
