use super::*;
use crate::subgraph::order::{GetOrderVaultsVolumeResult, GetOrdersResult, OrderWithSortedVaults};
use rain_orderbook_common::types::OrderDetailExtended;
use rain_orderbook_subgraph_client::{
    performance::OrderPerformance,
    types::common::{SgOrder, SgOrdersListFilterArgs},
    SgPaginationArgs,
};
use wasm_bindgen_utils::wasm_export;

const DEFAULT_PAGE_SIZE: u16 = 50;

#[wasm_export]
impl RaindexClient {
    /// Fetches orders with a given network or all networks.
    ///
    /// This method wraps the original [`get_orders`](crate::subgraph::order::get_orders) function,
    /// automatically resolving the appropriate subgraph URLs based on the provided chain ID.
    ///
    /// # Parameters
    ///
    /// * `chain_id` - Optional chain ID. If present, queries that specific network.
    ///   If not present, queries all configured networks.
    /// * `filter_args` - Optional filtering criteria (owners, active status, order hash)
    /// * `page` - Optional page number (1-based, defaults to 1)
    ///
    /// # Returns
    ///
    /// * `Ok(GetOrdersResult)` - Array of orders with network information
    /// * `Err(RaindexError)` - Configuration or network errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Query all networks
    /// const allOrders = await client.getOrders();
    ///
    /// // Query specific network
    /// const polygonOrders = await client.getOrders(137);
    ///
    /// // With filtering
    /// const activeOrders = await client.getOrders(137, { active: true });
    /// ```
    #[wasm_export(js_name = "getOrders", unchecked_return_type = "GetOrdersResult")]
    pub async fn get_orders(
        &self,
        chain_id: Option<u64>,
        filter_args: Option<SgOrdersListFilterArgs>,
        page: Option<u16>,
    ) -> Result<GetOrdersResult, RaindexError> {
        let multi_subgraph_args = self.get_multi_subgraph_args(chain_id)?;
        Ok(crate::subgraph::order::get_orders(
            multi_subgraph_args,
            filter_args.unwrap_or(SgOrdersListFilterArgs {
                owners: vec![],
                active: None,
                order_hash: None,
            }),
            SgPaginationArgs {
                page: page.unwrap_or(1),
                page_size: DEFAULT_PAGE_SIZE,
            },
        )
        .await?)
    }

    /// Fetches a specific order by hash with a given network.
    ///
    /// This method wraps the original [`get_order_by_hash`](crate::subgraph::order::get_order_by_hash) function,
    /// automatically resolving the subgraph URL from the chain ID.
    ///
    /// # Parameters
    ///
    /// * `chain_id` - Target network's chain ID
    /// * `order_hash` - Order hash identifier
    ///
    /// # Returns
    ///
    /// * `Ok(OrderWithSortedVaults)` - Order with categorized vaults
    /// * `Err(RaindexError)` - Order not found, network, or configuration errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const order = await client.getOrderByHash(
    ///   137,
    ///   "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    /// );
    /// ```
    #[wasm_export(
        js_name = "getOrderByHash",
        unchecked_return_type = "OrderWithSortedVaults"
    )]
    pub async fn get_order_by_hash(
        &self,
        chain_id: u64,
        order_hash: String,
    ) -> Result<OrderWithSortedVaults, RaindexError> {
        let subgraph_url = self.get_subgraph_url_for_chain(chain_id)?;
        Ok(crate::subgraph::order::get_order_by_hash(&subgraph_url, &order_hash).await?)
    }

    /// Extends order data with additional information.
    ///
    /// This method wraps the original [`order_detail_extended`](crate::subgraph::order::order_detail_extended) function.
    ///
    /// # Parameters
    ///
    /// * `order` - Base order object from subgraph
    ///
    /// # Returns
    ///
    /// * `Ok(OrderDetailExtended)` - Enhanced order with computed fields
    /// * `Err(RaindexError)` - Processing or validation errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const extendedOrder = client.extendOrder(rawOrder);
    /// ```
    #[wasm_export(js_name = "extendOrder", unchecked_return_type = "OrderDetailExtended")]
    pub fn extend_order(&self, order: SgOrder) -> Result<OrderDetailExtended, RaindexError> {
        Ok(crate::subgraph::order::order_detail_extended(order)?)
    }

    /// Calculates trading volume for an order's vaults.
    ///
    /// This method wraps the original [`order_vaults_volume`](crate::subgraph::order::order_vaults_volume) function,
    /// automatically resolving the subgraph URL from the chain ID.
    ///
    /// # Parameters
    ///
    /// * `chain_id` - Target network's chain ID
    /// * `order_id` - Order hash identifier
    /// * `start_timestamp` - Optional start time (Unix timestamp in seconds)
    /// * `end_timestamp` - Optional end time (Unix timestamp in seconds)
    ///
    /// # Returns
    ///
    /// * `Ok(GetOrderVaultsVolumeResult)` - Array of vault volume statistics
    /// * `Err(RaindexError)` - Calculation, network, or configuration errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const thirtyDaysAgo = Math.floor(Date.now() / 1000) - (30 * 24 * 60 * 60);
    /// const volumes = await client.getOrderVaultsVolume(137, orderId, thirtyDaysAgo);
    /// ```
    #[wasm_export(
        js_name = "getOrderVaultsVolume",
        unchecked_return_type = "GetOrderVaultsVolumeResult"
    )]
    pub async fn get_order_vaults_volume(
        &self,
        chain_id: u64,
        order_id: String,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<GetOrderVaultsVolumeResult, RaindexError> {
        let subgraph_url = self.get_subgraph_url_for_chain(chain_id)?;
        Ok(crate::subgraph::order::order_vaults_volume(
            &subgraph_url,
            &order_id,
            start_timestamp,
            end_timestamp,
        )
        .await?)
    }

    /// Analyzes comprehensive performance metrics for an order.
    ///
    /// This method wraps the original [`order_performance`](crate::subgraph::order::order_performance) function,
    /// automatically resolving the subgraph URL from the chain ID.
    ///
    /// # Parameters
    ///
    /// * `chain_id` - Target network's chain ID
    /// * `order_id` - Order hash identifier
    /// * `start_timestamp` - Optional start time (Unix timestamp in seconds)
    /// * `end_timestamp` - Optional end time (Unix timestamp in seconds)
    ///
    /// # Returns
    ///
    /// * `Ok(OrderPerformance)` - Comprehensive performance metrics including vaults APY, volume, and total metrics
    /// * `Err(RaindexError)` - Calculation, network, or configuration errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const weekAgo = Math.floor(Date.now() / 1000) - (7 * 24 * 60 * 60);
    /// const metrics = await client.getOrderPerformance(137, orderId, weekAgo);
    /// ```
    #[wasm_export(
        js_name = "getOrderPerformance",
        unchecked_return_type = "OrderPerformance"
    )]
    pub async fn get_order_performance(
        &self,
        chain_id: u64,
        order_id: String,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<OrderPerformance, RaindexError> {
        let subgraph_url = self.get_subgraph_url_for_chain(chain_id)?;
        Ok(crate::subgraph::order::order_performance(
            &subgraph_url,
            &order_id,
            start_timestamp,
            end_timestamp,
        )
        .await?)
    }
}
