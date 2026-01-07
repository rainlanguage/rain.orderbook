use super::local_db::orders::LocalDbOrders;
use super::trades::RaindexTrade;
use super::*;
use crate::local_db::query::fetch_orders::LocalDbOrder;
use crate::local_db::query::fetch_vaults::LocalDbVault;
use crate::local_db::OrderbookIdentifier;
use crate::raindex_client::vaults_list::RaindexVaultsList;
use crate::{
    meta::TryDecodeRainlangSource,
    raindex_client::{
        transactions::RaindexTransaction,
        vaults::{RaindexVault, RaindexVaultType},
    },
};
use alloy::primitives::{b256, keccak256, Address, Bytes, B256, U256};
use async_trait::async_trait;
use csv::{ReaderBuilder, Terminator};
use rain_orderbook_subgraph_client::{
    types::{
        common::{
            SgBigInt, SgBytes, SgOrder, SgOrderAsIO, SgOrderbook, SgOrdersListFilterArgs, SgVault,
        },
        Id,
    },
    MultiOrderbookSubgraphClient, OrderbookSubgraphClient, OrderbookSubgraphClientError,
    SgPaginationArgs,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Cursor, rc::Rc, str::FromStr};
use tsify::Tsify;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::BigInt;

const DEFAULT_PAGE_SIZE: u16 = 100;

pub(crate) struct SubgraphOrders<'a> {
    client: &'a RaindexClient,
}
impl<'a> SubgraphOrders<'a> {
    pub(crate) fn new(client: &'a RaindexClient) -> Self {
        Self { client }
    }
}

#[async_trait(?Send)]
pub(crate) trait OrdersDataSource {
    async fn list(
        &self,
        chain_ids: Option<Vec<u32>>,
        filters: &GetOrdersFilters,
        page: Option<u16>,
    ) -> Result<Vec<RaindexOrder>, RaindexError>;

    async fn get_by_hash(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
    ) -> Result<Option<RaindexOrder>, RaindexError>;

    async fn get_added_by_tx_hash(
        &self,
        chain_id: u32,
        orderbook: Address,
        tx_hash: B256,
    ) -> Result<Vec<RaindexOrder>, RaindexError>;

    async fn get_removed_by_tx_hash(
        &self,
        chain_id: u32,
        orderbook: Address,
        tx_hash: B256,
    ) -> Result<Vec<RaindexOrder>, RaindexError>;

    async fn trades_list(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
        page: Option<u16>,
    ) -> Result<Vec<RaindexTrade>, RaindexError>;

    async fn trades_count(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<u64, RaindexError>;
}

/// A single order representation within a given orderbook.
///
/// RaindexOrder represents a trading order on a specific blockchain with its associated
/// input and output vaults, metadata, and performance tracking capabilities. Each order
/// is deployed on a specific orderbook contract and can be queried for volume and
/// performance metrics over time.
///
/// The order contains both the raw order data (bytes and hash) and structured access
/// to its vaults, which define what tokens can be traded and their current balances.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexOrder {
    raindex_client: Rc<RaindexClient>,
    chain_id: u32,
    id: B256,
    order_bytes: Bytes,
    order_hash: B256,
    owner: Address,
    inputs: Vec<RaindexVault>,
    outputs: Vec<RaindexVault>,
    orderbook: Address,
    active: bool,
    timestamp_added: U256,
    meta: Option<Bytes>,
    rainlang: Option<String>,
    transaction: Option<RaindexTransaction>,
    trades_count: u16,
}

fn get_io_by_type(order: &RaindexOrder, vault_type: RaindexVaultType) -> Vec<RaindexVault> {
    let vaults = order.vaults_list().items();
    vaults
        .into_iter()
        .filter(|v| v.vault_type() == Some(vault_type.clone()))
        .collect()
}

impl RaindexOrder {
    pub(crate) fn from_local_db_order(
        raindex_client: Rc<RaindexClient>,
        order: LocalDbOrder,
        inputs: Vec<LocalDbVault>,
        outputs: Vec<LocalDbVault>,
    ) -> Result<Self, RaindexError> {
        let chain_id = order.chain_id;
        let rainlang = order
            .meta
            .as_ref()
            .and_then(|meta| meta.to_string().try_decode_rainlangsource().ok());

        let mut id = Vec::from(order.orderbook_address.as_slice());
        id.extend_from_slice(order.order_hash.as_ref());

        Ok(Self {
            raindex_client: Rc::clone(&raindex_client),
            chain_id,
            id: keccak256(&id),
            order_bytes: order.order_bytes,
            order_hash: order.order_hash,
            owner: order.owner,
            inputs: inputs
                .into_iter()
                .map(|v| {
                    RaindexVault::try_from_local_db(
                        Rc::clone(&raindex_client),
                        v,
                        Some(RaindexVaultType::Input),
                    )
                })
                .collect::<Result<Vec<RaindexVault>, RaindexError>>()?,
            outputs: outputs
                .into_iter()
                .map(|v| {
                    RaindexVault::try_from_local_db(
                        Rc::clone(&raindex_client),
                        v,
                        Some(RaindexVaultType::Output),
                    )
                })
                .collect::<Result<Vec<RaindexVault>, RaindexError>>()?,
            orderbook: order.orderbook_address,
            active: order.active,
            timestamp_added: U256::from(order.block_timestamp),
            meta: order.meta,
            rainlang,
            transaction: Some(RaindexTransaction::from_local_parts(
                order.transaction_hash,
                order.owner,
                order.block_number,
                order.block_timestamp,
            )?),
            trades_count: order.trade_count as u16,
        })
    }
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexOrder {
    #[wasm_bindgen(getter = chainId)]
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Hex")]
    pub fn id(&self) -> String {
        self.id.to_string()
    }
    #[wasm_bindgen(getter = orderBytes, unchecked_return_type = "Hex")]
    pub fn order_bytes(&self) -> String {
        self.order_bytes.to_string()
    }
    #[wasm_bindgen(getter = orderHash, unchecked_return_type = "Hex")]
    pub fn order_hash(&self) -> String {
        self.order_hash.to_string()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn owner(&self) -> String {
        self.owner.to_string()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn active(&self) -> bool {
        self.active
    }
    #[wasm_bindgen(getter = timestampAdded)]
    pub fn timestamp_added(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.timestamp_added.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Hex | undefined")]
    pub fn meta(&self) -> Option<String> {
        self.meta.clone().map(|meta| meta.to_string())
    }
    #[wasm_bindgen(getter)]
    pub fn rainlang(&self) -> Option<String> {
        self.rainlang.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn transaction(&self) -> Option<RaindexTransaction> {
        self.transaction.clone()
    }
    #[wasm_bindgen(getter = tradesCount)]
    pub fn trades_count(&self) -> u16 {
        self.trades_count
    }

    #[wasm_bindgen(getter = vaultsList)]
    pub fn vaults_list(&self) -> RaindexVaultsList {
        RaindexVaultsList::new(get_vaults_with_type(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }
    #[wasm_bindgen(getter = inputsList)]
    pub fn inputs_list(&self) -> RaindexVaultsList {
        RaindexVaultsList::new(get_io_by_type(self, RaindexVaultType::Input))
    }
    #[wasm_bindgen(getter = outputsList)]
    pub fn outputs_list(&self) -> RaindexVaultsList {
        RaindexVaultsList::new(get_io_by_type(self, RaindexVaultType::Output))
    }
    #[wasm_bindgen(getter = inputsOutputsList)]
    pub fn inputs_outputs_list(&self) -> RaindexVaultsList {
        RaindexVaultsList::new(get_io_by_type(self, RaindexVaultType::InputOutput))
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexOrder {
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    pub fn id(&self) -> B256 {
        self.id
    }
    pub fn order_bytes(&self) -> Bytes {
        self.order_bytes.clone()
    }
    pub fn order_hash(&self) -> B256 {
        self.order_hash
    }
    pub fn owner(&self) -> Address {
        self.owner
    }
    pub fn orderbook(&self) -> Address {
        self.orderbook
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn timestamp_added(&self) -> U256 {
        self.timestamp_added
    }
    pub fn meta(&self) -> Option<Bytes> {
        self.meta.clone()
    }
    pub fn rainlang(&self) -> Option<String> {
        self.rainlang.clone()
    }
    pub fn transaction(&self) -> Option<RaindexTransaction> {
        self.transaction.clone()
    }
    pub fn trades_count(&self) -> u16 {
        self.trades_count
    }
    pub fn vaults_list(&self) -> RaindexVaultsList {
        RaindexVaultsList::new(get_vaults_with_type(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }
    pub fn inputs_list(&self) -> RaindexVaultsList {
        RaindexVaultsList::new(get_io_by_type(self, RaindexVaultType::Input))
    }
    pub fn outputs_list(&self) -> RaindexVaultsList {
        RaindexVaultsList::new(get_io_by_type(self, RaindexVaultType::Output))
    }
    pub fn inputs_outputs_list(&self) -> RaindexVaultsList {
        RaindexVaultsList::new(get_io_by_type(self, RaindexVaultType::InputOutput))
    }
}

fn get_vaults_with_type(
    inputs: Vec<RaindexVault>,
    outputs: Vec<RaindexVault>,
) -> Vec<RaindexVault> {
    let mut vaults: Vec<RaindexVault> = Vec::new();

    let input_ids: HashSet<String> = inputs.iter().map(|v| v.id().to_string()).collect();
    let output_ids: HashSet<String> = outputs.iter().map(|v| v.id().to_string()).collect();

    // First add inputs (excluding input_outputs)
    for vault in &inputs {
        if !output_ids.contains(&vault.id().to_string()) {
            vaults.push(vault.clone());
        }
    }
    // Then add outputs (excluding input_outputs)
    for vault in &outputs {
        if !input_ids.contains(&vault.id().to_string()) {
            vaults.push(vault.clone());
        }
    }
    // Finally add input_outputs (only once for vaults that are both input and output)
    for vault in &inputs {
        if output_ids.contains(&vault.id().to_string()) {
            let input_output_vault = vault.with_vault_type(RaindexVaultType::InputOutput);
            vaults.push(input_output_vault);
        }
    }
    vaults
}

#[wasm_export]
impl RaindexOrder {
    #[wasm_export(skip)]
    pub fn get_raindex_client(&self) -> Rc<RaindexClient> {
        Rc::clone(&self.raindex_client)
    }
    #[wasm_export(skip)]
    pub fn get_orderbook_client(&self) -> Result<OrderbookSubgraphClient, RaindexError> {
        self.raindex_client.get_orderbook_client(self.orderbook)
    }
    #[wasm_export(skip)]
    pub fn get_rpc_urls(&self) -> Result<Vec<Url>, RaindexError> {
        self.raindex_client.get_rpc_urls_for_chain(self.chain_id)
    }

    // /// Retrieves volume data for all vaults associated with this order over a specified time period
    // ///
    // /// Queries historical volume information across all vaults that belong to this order,
    // /// allowing analysis of trading activity and liquidity patterns over time.
    // ///
    // /// ## Examples
    // ///
    // /// ```javascript
    // /// const result = await order.getVaultsVolume(
    // ///   Math.floor(Date.now() / 1000) - 86400, // 24 hours ago
    // ///   Math.floor(Date.now() / 1000)
    // /// );
    // /// if (result.error) {
    // ///   console.error("Error fetching volume:", result.error.readableMsg);
    // ///   return;
    // /// }
    // /// const volumes = result.value;
    // /// // Do something with volumes
    // /// ```
    // #[wasm_export(
    //     js_name = "getVaultsVolume",
    //     return_description = "Volume data for each vault over the specified period",
    //     unchecked_return_type = "RaindexVaultVolume[]",
    //     preserve_js_class
    // )]
    // pub async fn get_vaults_volume(
    //     &self,
    //     #[wasm_export(
    //         js_name = "startTimestamp",
    //         param_description = "Unix timestamp for the start of the query period (optional)"
    //     )]
    //     start_timestamp: Option<u64>,
    //     #[wasm_export(
    //         js_name = "endTimestamp",
    //         param_description = "Unix timestamp for the end of the query period (optional)"
    //     )]
    //     end_timestamp: Option<u64>,
    // ) -> Result<Vec<RaindexVaultVolume>, RaindexError> {
    //     let client = self.get_orderbook_client()?;

    //     let mut result_volumes = Vec::new();
    //     let volumes = client
    //         .order_vaults_volume(Id::new(self.id.to_string()), start_timestamp, end_timestamp)
    //         .await?;
    //     for volume in volumes {
    //         let volume = RaindexVaultVolume::try_from_vault_volume(self.chain_id, volume)?;
    //         result_volumes.push(volume);
    //     }
    //     Ok(result_volumes)
    // }

    // /// Gets comprehensive performance metrics and analytics for this order over a specified time period
    // ///
    // /// Retrieves detailed performance data including profit/loss, volume statistics, and other
    // /// key metrics that help assess the effectiveness of the trading algorithm implemented by this order.
    // ///
    // /// ## Examples
    // ///
    // /// ```javascript
    // /// const result = await order.getPerformance(
    // ///   Math.floor(Date.now() / 1000) - 604800, // 1 week ago
    // ///   Math.floor(Date.now() / 1000)
    // /// );
    // /// if (result.error) {
    // ///   console.error("Error fetching performance:", result.error.readableMsg);
    // ///   return;
    // /// }
    // /// const performance = result.value;
    // /// // Do something with performance
    // /// ```
    // #[wasm_export(
    //     js_name = "getPerformance",
    //     return_description = "Comprehensive performance metrics for the order",
    //     unchecked_return_type = "OrderPerformance"
    // )]
    // pub async fn get_performance(
    //     &self,
    //     #[wasm_export(
    //         js_name = "startTimestamp",
    //         param_description = "Unix timestamp for the start of the analysis period (optional, defaults to order creation)"
    //     )]
    //     start_timestamp: Option<u64>,
    //     #[wasm_export(
    //         js_name = "endTimestamp",
    //         param_description = "Unix timestamp for the end of the analysis period (optional, defaults to current time)"
    //     )]
    //     end_timestamp: Option<u64>,
    // ) -> Result<OrderPerformance, RaindexError> {
    //     let client = self.get_orderbook_client()?;
    //     let performance = client
    //         .order_performance(Id::new(self.id.to_string()), start_timestamp, end_timestamp)
    //         .await?;
    //     Ok(performance)
    // }

    /// Converts the order from RaindexOrder to an SgOrder type
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const sgOrder = await order.convertToSgOrder();
    /// // Do something with sgOrder
    /// ```
    #[wasm_export(
        js_name = "convertToSgOrder",
        return_description = "Order as SgOrder type",
        unchecked_return_type = "SgOrder"
    )]
    pub fn convert_to_sg_order(&self) -> Result<SgOrder, RaindexError> {
        let sg_order = self.clone().into_sg_order()?;
        Ok(sg_order)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RaindexOrderAsIO {
    #[tsify(type = "Hex")]
    pub id: B256,
    #[tsify(type = "Hex")]
    pub order_hash: B256,
    pub active: bool,
}
impl_wasm_traits!(RaindexOrderAsIO);
impl TryFrom<SgOrderAsIO> for RaindexOrderAsIO {
    type Error = RaindexError;
    fn try_from(order: SgOrderAsIO) -> Result<Self, Self::Error> {
        Ok(Self {
            id: B256::from_str(&order.id.0)?,
            order_hash: B256::from_str(&order.order_hash.0)?,
            active: order.active,
        })
    }
}
impl TryFrom<RaindexOrderAsIO> for SgOrderAsIO {
    type Error = RaindexError;
    fn try_from(order: RaindexOrderAsIO) -> Result<Self, Self::Error> {
        Ok(Self {
            id: SgBytes(order.id.to_string()),
            order_hash: SgBytes(order.order_hash.to_string()),
            active: order.active,
        })
    }
}

impl RaindexOrderAsIO {
    pub fn try_from_local_db_orders_csv(
        field_name: &str,
        csv: &Option<String>,
    ) -> Result<Vec<RaindexOrderAsIO>, RaindexError> {
        let mut result = Vec::new();
        let Some(csv_str) = csv.as_ref() else {
            return Ok(result);
        };
        if csv_str.is_empty() {
            return Ok(result);
        }

        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b':')
            .terminator(Terminator::Any(b','))
            .from_reader(Cursor::new(csv_str.as_bytes()));

        for record in reader.records() {
            let record = record.map_err(|err| {
                RaindexError::JsError(format!(
                    "Invalid {} entry: failed to parse record ({err})",
                    field_name
                ))
            })?;
            let mut fields = record.iter();
            let _id_str = fields.next().ok_or(RaindexError::JsError(format!(
                "Invalid {} entry: missing id",
                field_name
            )))?;
            let hash_str = fields.next().ok_or(RaindexError::JsError(format!(
                "Invalid {} entry: missing order hash",
                field_name
            )))?;
            let active_str = fields.next().ok_or(RaindexError::JsError(format!(
                "Invalid {} entry: missing active flag",
                field_name
            )))?;
            if fields.next().is_some() {
                return Err(RaindexError::JsError(format!(
                    "Invalid {} entry: too many fields",
                    field_name
                )));
            }
            let order_hash = B256::from_str(hash_str)?;
            let active = match active_str {
                "1" => true,
                "0" => false,
                _ => {
                    return Err(RaindexError::JsError(format!(
                        "Invalid active flag in {}: {}",
                        field_name, active_str
                    )))
                }
            };
            result.push(RaindexOrderAsIO {
                id: b256!("0x0000000000000000000000000000000000000000000000000000000000000001"),
                order_hash,
                active,
            });
        }
        Ok(result)
    }
}

#[wasm_export]
impl RaindexClient {
    /// Queries orders with filtering and pagination across configured networks
    ///
    /// Retrieves a list of orders from the specified network or all configured networks,
    /// with support for filtering by owner, status, and order hash. Results are paginated
    /// for efficient data retrieval.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getOrders(
    ///   137, // Polygon network
    ///   {
    ///     owners: ["0x1234567890abcdef1234567890abcdef12345678"],
    ///     active: true
    ///   },
    ///   1
    /// );
    /// if (result.error) {
    ///   console.error("Error fetching orders:", result.error.readableMsg);
    ///   return;
    /// }
    /// const orders = result.value;
    /// // Do something with orders
    /// ```
    #[wasm_export(
        js_name = "getOrders",
        return_description = "Array of raindex order instances",
        unchecked_return_type = "RaindexOrder[]",
        preserve_js_class
    )]
    pub async fn get_orders(
        &self,
        #[wasm_export(
            js_name = "chainIds",
            param_description = "Specific blockchain network to query (optional, queries all networks if not specified)"
        )]
        chain_ids: Option<ChainIds>,
        #[wasm_export(
            param_description = "Filtering criteria including owners, active status, and order hash (optional)"
        )]
        filters: Option<GetOrdersFilters>,
        #[wasm_export(param_description = "Page number for pagination (optional, defaults to 1)")]
        page: Option<u16>,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        let filters = filters.unwrap_or_default();
        let page_number = page.unwrap_or(1);
        let ids = chain_ids.map(|ChainIds(ids)| ids);

        if let Some(local_db) = self.local_db() {
            let local_source = LocalDbOrders::new(&local_db, Rc::new(self.clone()));
            return local_source.list(ids, &filters, None).await;
        }

        let subgraph_source = SubgraphOrders::new(self);
        subgraph_source.list(ids, &filters, Some(page_number)).await
    }

    /// Retrieves a specific order by its hash from a particular blockchain network
    ///
    /// Fetches complete order details including all vault information, metadata, and
    /// performance tracking capabilities for a specific order identified by its hash.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getOrderByHash(
    ///   137, // Polygon network
    ///   "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    /// );
    /// if (result.error) {
    ///   console.error("Error fetching order:", result.error.readableMsg);
    ///   return;
    /// }
    /// const order = result.value;
    /// // Do something with order
    /// ```
    #[wasm_export(
        js_name = "getOrderByHash",
        return_description = "Complete order details with vault and metadata information",
        unchecked_return_type = "RaindexOrder",
        preserve_js_class
    )]
    pub async fn get_order_by_hash_wasm_binding(
        &self,
        #[wasm_export(
            js_name = "chainId",
            param_description = "The blockchain network ID where the order exists"
        )]
        chain_id: u32,
        #[wasm_export(
            js_name = "orderbookAddress",
            param_description = "Orderbook contract address",
            unchecked_param_type = "Address"
        )]
        orderbook_address: String,
        #[wasm_export(
            js_name = "orderHash",
            param_description = "The unique hash identifier of the order",
            unchecked_param_type = "Hex"
        )]
        order_hash: String,
    ) -> Result<RaindexOrder, RaindexError> {
        let orderbook_address = Address::from_str(&orderbook_address)?;
        let order_hash = B256::from_str(&order_hash)?;
        self.get_order_by_hash(
            &OrderbookIdentifier::new(chain_id, orderbook_address),
            order_hash,
        )
        .await
    }
}

#[async_trait(?Send)]
impl OrdersDataSource for SubgraphOrders<'_> {
    async fn list(
        &self,
        chain_ids: Option<Vec<u32>>,
        filters: &GetOrdersFilters,
        page: Option<u16>,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        let raindex_client = Rc::new(self.client.clone());
        let multi_subgraph_args = self.client.get_multi_subgraph_args(chain_ids)?;

        let client = MultiOrderbookSubgraphClient::new(
            multi_subgraph_args.values().flatten().cloned().collect(),
        );

        let orders = client
            .orders_list(
                filters.clone().try_into()?,
                SgPaginationArgs {
                    page: page.unwrap_or(1),
                    page_size: DEFAULT_PAGE_SIZE,
                },
            )
            .await;

        let orders = orders
            .iter()
            .map(|order| {
                let chain_id = multi_subgraph_args
                    .iter()
                    .find(|(_, args)| args.iter().any(|arg| arg.name == order.subgraph_name))
                    .map(|(chain_id, _)| *chain_id)
                    .ok_or(RaindexError::SubgraphNotFound(
                        order.subgraph_name.clone(),
                        order.order.order_hash.0.clone(),
                    ))?;
                RaindexOrder::try_from_sg_order(
                    raindex_client.clone(),
                    chain_id,
                    order.order.clone(),
                    None,
                )
            })
            .collect::<Result<Vec<RaindexOrder>, RaindexError>>()?;

        Ok(orders)
    }

    async fn get_by_hash(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
    ) -> Result<Option<RaindexOrder>, RaindexError> {
        let raindex_client = Rc::new(self.client.clone());
        let client = self.client.get_orderbook_client(ob_id.orderbook_address)?;
        let order = match client
            .order_detail_by_hash(SgBytes(order_hash.to_string()))
            .await
        {
            Ok(order) => order,
            Err(OrderbookSubgraphClientError::Empty) => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let order = RaindexOrder::try_from_sg_order(raindex_client, ob_id.chain_id, order, None)?;
        Ok(Some(order))
    }

    async fn get_added_by_tx_hash(
        &self,
        chain_id: u32,
        orderbook: Address,
        tx_hash: B256,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        let raindex_client = Rc::new(self.client.clone());
        let client = self.client.get_orderbook_client(orderbook)?;
        let sg_orders = client
            .transaction_add_orders(Id::new(tx_hash.to_string()))
            .await?;
        sg_orders
            .into_iter()
            .map(|value| {
                let transaction = value.transaction.try_into()?;
                RaindexOrder::try_from_sg_order(
                    raindex_client.clone(),
                    chain_id,
                    value.order,
                    Some(transaction),
                )
            })
            .collect()
    }

    async fn get_removed_by_tx_hash(
        &self,
        chain_id: u32,
        orderbook: Address,
        tx_hash: B256,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        let raindex_client = Rc::new(self.client.clone());
        let client = self.client.get_orderbook_client(orderbook)?;
        let sg_orders = client
            .transaction_remove_orders(Id::new(tx_hash.to_string()))
            .await?;
        sg_orders
            .into_iter()
            .map(|value| {
                let transaction = value.transaction.try_into()?;
                RaindexOrder::try_from_sg_order(
                    raindex_client.clone(),
                    chain_id,
                    value.order,
                    Some(transaction),
                )
            })
            .collect()
    }

    async fn trades_list(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
        page: Option<u16>,
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let client = self.client.get_orderbook_client(ob_id.orderbook_address)?;

        let order = client
            .order_detail_by_hash(SgBytes(order_hash.to_string()))
            .await?;

        let trades = client
            .order_trades_list(
                Id::new(order.id.0.clone()),
                SgPaginationArgs {
                    page: page.unwrap_or(1),
                    page_size: DEFAULT_PAGE_SIZE,
                },
                start_timestamp,
                end_timestamp,
            )
            .await?;

        trades
            .into_iter()
            .map(|trade| RaindexTrade::try_from_sg_trade(ob_id.chain_id, trade))
            .collect()
    }

    async fn trades_count(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<u64, RaindexError> {
        let client = self.client.get_orderbook_client(ob_id.orderbook_address)?;

        let order = client
            .order_detail_by_hash(SgBytes(order_hash.to_string()))
            .await?;

        let trades = client
            .order_trades_list_all(Id::new(order.id.0.clone()), start_timestamp, end_timestamp)
            .await?;

        Ok(trades.len() as u64)
    }
}

impl RaindexClient {
    pub async fn get_order_by_hash(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: B256,
    ) -> Result<RaindexOrder, RaindexError> {
        let orderbook_cfg = self.get_orderbook_by_address(ob_id.orderbook_address)?;
        if orderbook_cfg.network.chain_id != ob_id.chain_id {
            return Err(RaindexError::OrderbookNotFound(
                ob_id.orderbook_address.to_string(),
                ob_id.chain_id,
            ));
        }

        if let Some(local_db) = self.local_db() {
            let local_source = LocalDbOrders::new(&local_db, Rc::new(self.clone()));
            if let Some(order) = local_source.get_by_hash(ob_id, &order_hash).await? {
                return Ok(order);
            }
        }

        SubgraphOrders::new(self)
            .get_by_hash(ob_id, &order_hash)
            .await?
            .ok_or_else(|| {
                RaindexError::OrderNotFound(
                    ob_id.orderbook_address.to_string(),
                    ob_id.chain_id,
                    order_hash,
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetOrdersFilters {
    #[tsify(type = "Address[]")]
    pub owners: Vec<Address>,
    #[tsify(optional)]
    pub active: Option<bool>,
    #[tsify(optional, type = "Hex")]
    pub order_hash: Option<B256>,
    #[tsify(optional, type = "Address[]")]
    pub tokens: Option<Vec<Address>>,
}
impl_wasm_traits!(GetOrdersFilters);

impl TryFrom<GetOrdersFilters> for SgOrdersListFilterArgs {
    type Error = RaindexError;
    fn try_from(filters: GetOrdersFilters) -> Result<Self, Self::Error> {
        Ok(Self {
            owners: filters
                .owners
                .into_iter()
                .map(|owner| SgBytes(owner.to_string()))
                .collect(),
            active: filters.active,
            order_hash: filters
                .order_hash
                .map(|order_hash| SgBytes(order_hash.to_string())),
            tokens: filters
                .tokens
                .map(|tokens| {
                    tokens
                        .into_iter()
                        .map(|token| token.to_string().to_lowercase())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl RaindexOrder {
    pub fn try_from_sg_order(
        raindex_client: Rc<RaindexClient>,
        chain_id: u32,
        order: SgOrder,
        transaction: Option<RaindexTransaction>,
    ) -> Result<Self, RaindexError> {
        let rainlang = order
            .meta
            .as_ref()
            .and_then(|meta| meta.0.try_decode_rainlangsource().ok());

        Ok(Self {
            raindex_client: Rc::clone(&raindex_client),
            chain_id,
            id: B256::from_str(&order.id.0)?,
            order_bytes: Bytes::from_str(&order.order_bytes.0)?,
            order_hash: B256::from_str(&order.order_hash.0)?,
            owner: Address::from_str(&order.owner.0)?,
            inputs: {
                order
                    .inputs
                    .iter()
                    .map(|v| {
                        RaindexVault::try_from_sg_vault(
                            Rc::clone(&raindex_client),
                            chain_id,
                            v.clone(),
                            Some(RaindexVaultType::Input),
                        )
                    })
                    .collect::<Result<Vec<RaindexVault>, RaindexError>>()?
            },
            outputs: {
                order
                    .outputs
                    .iter()
                    .map(|v| {
                        RaindexVault::try_from_sg_vault(
                            Rc::clone(&raindex_client),
                            chain_id,
                            v.clone(),
                            Some(RaindexVaultType::Output),
                        )
                    })
                    .collect::<Result<Vec<RaindexVault>, RaindexError>>()?
            },
            orderbook: Address::from_str(&order.orderbook.id.0)?,
            active: order.active,
            timestamp_added: U256::from_str(&order.timestamp_added.0)?,
            meta: order
                .meta
                .map(|meta| Bytes::from_str(&meta.0))
                .transpose()?,
            rainlang,
            transaction,
            trades_count: order.trades.len() as u16,
        })
    }

    pub fn into_sg_order(self) -> Result<SgOrder, RaindexError> {
        #[cfg(target_family = "wasm")]
        let timestamp_added = self.timestamp_added.to_string();
        #[cfg(not(target_family = "wasm"))]
        let timestamp_added = self.timestamp_added().to_string();

        Ok(SgOrder {
            id: SgBytes(self.id().to_string()),
            order_bytes: SgBytes(self.order_bytes().to_string()),
            order_hash: SgBytes(self.order_hash().to_string()),
            owner: SgBytes(self.owner().to_string()),
            outputs: self
                .outputs
                .clone()
                .into_iter()
                .map(|v| v.into_sg_vault())
                .collect::<Result<Vec<SgVault>, RaindexError>>()?,
            inputs: self
                .inputs
                .clone()
                .into_iter()
                .map(|v| v.into_sg_vault())
                .collect::<Result<Vec<SgVault>, RaindexError>>()?,
            orderbook: SgOrderbook {
                id: SgBytes(self.orderbook().to_string()),
            },
            active: self.active(),
            timestamp_added: SgBigInt(timestamp_added),
            meta: self.meta().map(|meta| SgBytes(meta.to_string())),
            add_events: vec![],
            remove_events: vec![],
            trades: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use crate::{
            local_db::query::{
                fetch_orders::LocalDbOrder, FromDbJson, LocalDbQueryError, LocalDbQueryExecutor,
                SqlStatement, SqlStatementBatch,
            },
            raindex_client::local_db::LocalDb,
        };
        use alloy::primitives::{b256, U256};
        use httpmock::MockServer;
        use rain_math_float::Float;
        use rain_orderbook_subgraph_client::utils::float::*;
        use rain_orderbook_subgraph_client::{
            // performance::{
            //     apy::APYDetails, vol::VolumeDetails, DenominatedPerformance, VaultPerformance,
            // },
            types::common::{
                SgAddOrder, SgBigInt, SgBytes, SgErc20, SgOrderAsIO, SgOrderbook, SgTransaction,
                SgVault,
            },
        };
        use serde_json::{json, Value};
        use std::str::FromStr;

        #[derive(Clone)]
        struct StaticJsonExec {
            json: String,
        }

        #[async_trait(?Send)]
        impl LocalDbQueryExecutor for StaticJsonExec {
            async fn execute_batch(
                &self,
                _batch: &SqlStatementBatch,
            ) -> Result<(), LocalDbQueryError> {
                Ok(())
            }

            async fn query_json<T>(&self, _stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
            where
                T: FromDbJson,
            {
                serde_json::from_str(&self.json)
                    .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
            }

            async fn query_text(&self, _stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
                Err(LocalDbQueryError::database(
                    "query_text not supported in StaticJsonExec",
                ))
            }
        }

        #[test]
        fn try_from_local_db_orders_csv_parses_records() {
            let csv = Some(
                "0xdeadbeef:0xabc0000000000000000000000000000000000000000000000000000000000001:1,\
                 0xdeadbeee:0xabc0000000000000000000000000000000000000000000000000000000000002:0"
                    .to_string(),
            );
            let parsed =
                RaindexOrderAsIO::try_from_local_db_orders_csv("inputOrders", &csv).unwrap();
            assert_eq!(parsed.len(), 2);
            assert_eq!(
                parsed[0].order_hash,
                B256::from_str(
                    "0xabc0000000000000000000000000000000000000000000000000000000000001"
                )
                .unwrap()
            );
            assert!(parsed[0].active);
            assert!(!parsed[1].active);
        }

        #[test]
        fn try_from_local_db_orders_csv_rejects_invalid_active_flag() {
            let csv = Some(
                "0xdeadbeef:0xabc0000000000000000000000000000000000000000000000000000000000001:maybe"
                    .to_string(),
            );
            let err =
                RaindexOrderAsIO::try_from_local_db_orders_csv("testField", &csv).unwrap_err();
            match err {
                RaindexError::JsError(msg) => {
                    assert!(msg.contains("Invalid active flag in testField: maybe"))
                }
                _ => panic!("expected JsError"),
            }
        }

        fn get_order1_json() -> Value {
            json!(                        {
              "id": "0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1",
              "orderBytes": "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000f08bcbce72f62c95dcb7c07dcb5ed26acfcfbc1100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000005c00000000000000000000000000000000000000000000000000000000000000640392c489ef67afdc348209452c338ea5ba2b6152b936e152f610d05e1a20621a40000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae60000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000049d000000000000000000000000000000000000000000000000000000000000000f0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000c7d713b49da0000914d696e20747261646520616d6f756e742e00000000000000000000000000008b616d6f756e742d75736564000000000000000000000000000000000000000000000000000000000000000000000000000000000000000340aad21b3b70000000000000000000000000000000000000000000000000006194049f30f7200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000e043da6172500008f6c6173742d74726164652d74696d65000000000000000000000000000000008d6c6173742d74726164652d696f0000000000000000000000000000000000008c696e697469616c2d74696d650000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b200000000000000000000000000000000000000000000000000008ac7230489e80000000000000000000000020000915e36ef882941816356bc3718df868054f868ad000000000000000000000000000000000000000000000000000000000000027d0a00000024007400e0015801b401e001f40218025c080500040b20000200100001001000000b120003001000010b110004001000030b0100051305000201100001011000003d120000011000020010000003100404211200001d02000001100003031000010c1200004911000003100404001000012b12000001100003031000010c1200004a0200001a0b00090b1000060b20000700100000001000011b1200001a10000047120000001000001a1000004712000001100000011000002e12000001100005011000042e120000001000053d12000001100004001000042e1200000010000601100005001000032e120000481200011d0b020a0010000001100000011000062713000001100003031000010c12000049110000001000030010000247120000001000010b110008001000050110000700100001201200001f12000001100000011000004712000000100006001000073d120000011000002b12000000100008001000043b120000160901080b1000070b10000901100008001000013d1200001b12000001100006001000013d1200000b100009001000033a120000001000040010000248120001001000000b110008001000053d12000000100006001000042b1200000a0401011a10000001100009031000010c1200004a020000001000000110000a031000010c1200004a020000040200010110000b031000010c120000491100000803000201100009031000010c120000491100000110000a031000010c12000049110000100c01030110000d001000002e1200000110000c3e1200000010000100100001001000010010000100100001001000010010000100100001001000013d1a0000020100010210000e3611000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33",
              "orderHash": "0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4",
              "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
              "outputs": [
                {
                  "id": "0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf",
                  "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                  "vaultId": "75486334982066122983501547829219246999490818941767825330875804445439814023987",
                  "balance": Float::parse("0.987".to_string()).unwrap(),
                  "token": {
                    "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "name": "Staked FLR",
                    "symbol": "sFLR",
                    "decimals": "18"
                  },
                  "orderbook": {
                    "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                  },
                  "ordersAsOutput": [
                    {
                      "id": "0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1",
                      "orderHash": "0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4",
                      "active": true
                    }
                  ],
                  "ordersAsInput": [],
                  "balanceChanges": []
                },
                {
                    "id": "0x0000000000000000000000000000000000000000",
                    "token": {
                      "id": "0x0000000000000000000000000000000000000000",
                      "address": "0x0000000000000000000000000000000000000000",
                      "name": "T1",
                      "symbol": "T1",
                      "decimals": "0"
                    },
                    "balance": F0,
                    "vaultId": "0",
                    "owner": "0x0000000000000000000000000000000000000000",
                    "ordersAsOutput": [],
                    "ordersAsInput": [],
                    "balanceChanges": [],
                    "orderbook": {
                      "id": "0x0000000000000000000000000000000000000000"
                    }
                  }
              ],
              "inputs": [
                {
                  "id": "0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac",
                  "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                  "vaultId": "75486334982066122983501547829219246999490818941767825330875804445439814023987",
                  "balance": Float::parse("0.79799".to_string()).unwrap(),
                  "token": {
                    "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "name": "Wrapped Flare",
                    "symbol": "WFLR",
                    "decimals": "18"
                  },
                  "orderbook": {
                    "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                  },
                  "ordersAsOutput": [],
                  "ordersAsInput": [
                    {
                      "id": "0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1",
                      "orderHash": "0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4",
                      "active": true
                    }
                  ],
                  "balanceChanges": []
                },
                {
                    "id": "0x0000000000000000000000000000000000000000",
                    "token": {
                      "id": "0x0000000000000000000000000000000000000000",
                      "address": "0x0000000000000000000000000000000000000000",
                      "name": "T1",
                      "symbol": "T1",
                      "decimals": "0"
                    },
                    "balance": F0,
                    "vaultId": "0",
                    "owner": "0x0000000000000000000000000000000000000000",
                    "ordersAsOutput": [],
                    "ordersAsInput": [],
                    "balanceChanges": [],
                    "orderbook": {
                      "id": "0x0000000000000000000000000000000000000000"
                    }
                  }
              ],
              "orderbook": {
                "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
              },
              "active": true,
              "timestampAdded": "1739448802",
              "meta": "0xff0a89c674ee7874a3005902252f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d203078466532343131434461313933443945346538334135633233344337466433323031303138383361430a616d743a203130302c0a696f3a2063616c6c3c323e28293b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a63616c6c3c333e28292c0a3a656e7375726528657175616c2d746f286f75747075742d7661756c742d64656372656173652829203130302920226d7573742074616b652066756c6c20616d6f756e7422293b0a0a2f2a20322e206765742d696f2d726174696f2d6e6f77202a2f200a656c61707365643a2063616c6c3c343e28292c0a696f3a2073617475726174696e672d73756228302e3031373733353620646976286d756c28656c61707365642073756228302e3031373733353620302e30313733383434292920363029293b0a0a2f2a20332e206f6e652d73686f74202a2f200a3a656e737572652869732d7a65726f286765742868617368286f726465722d68617368282920226861732d657865637574656422292929202268617320657865637574656422292c0a3a7365742868617368286f726465722d68617368282920226861732d657865637574656422292031293b0a0a2f2a20342e206765742d656c6170736564202a2f200a5f3a20737562286e6f772829206765742868617368286f726465722d68617368282920226465706c6f792d74696d65222929293b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d",
              "addEvents": [
                {
                  "transaction": {
                    "id": "0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af",
                    "from": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                    "blockNumber": "37432554",
                    "timestamp": "1739448802"
                  }
                }
              ],
              "trades": [],
              "removeEvents": []
            })
        }

        fn get_order1() -> SgOrder {
            SgOrder {
                id: SgBytes("0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1".to_string()),
                order_bytes: SgBytes("0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000f08bcbce72f62c95dcb7c07dcb5ed26acfcfbc1100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000005c00000000000000000000000000000000000000000000000000000000000000640392c489ef67afdc348209452c338ea5ba2b6152b936e152f610d05e1a20621a40000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae60000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000049d000000000000000000000000000000000000000000000000000000000000000f0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000c7d713b49da0000914d696e20747261646520616d6f756e742e00000000000000000000000000008b616d6f756e742d75736564000000000000000000000000000000000000000000000000000000000000000000000000000000000000000340aad21b3b70000000000000000000000000000000000000000000000000006194049f30f7200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000e043da6172500008f6c6173742d74726164652d74696d65000000000000000000000000000000008d6c6173742d74726164652d696f0000000000000000000000000000000000008c696e697469616c2d74696d650000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b200000000000000000000000000000000000000000000000000008ac7230489e80000000000000000000000020000915e36ef882941816356bc3718df868054f868ad000000000000000000000000000000000000000000000000000000000000027d0a00000024007400e0015801b401e001f40218025c080500040b20000200100001001000000b120003001000010b110004001000030b0100051305000201100001011000003d120000011000020010000003100404211200001d02000001100003031000010c1200004911000003100404001000012b12000001100003031000010c1200004a0200001a0b00090b1000060b20000700100000001000011b1200001a10000047120000001000001a1000004712000001100000011000002e12000001100005011000042e120000001000053d12000001100004001000042e1200000010000601100005001000032e120000481200011d0b020a0010000001100000011000062713000001100003031000010c12000049110000001000030010000247120000001000010b110008001000050110000700100001201200001f12000001100000011000004712000000100006001000073d120000011000002b12000000100008001000043b120000160901080b1000070b10000901100008001000013d1200001b12000001100006001000013d1200000b100009001000033a120000001000040010000248120001001000000b110008001000053d12000000100006001000042b1200000a0401011a10000001100009031000010c1200004a020000001000000110000a031000010c1200004a020000040200010110000b031000010c120000491100000803000201100009031000010c120000491100000110000a031000010c12000049110000100c01030110000d001000002e1200000110000c3e1200000010000100100001001000010010000100100001001000010010000100100001001000013d1a0000020100010210000e3611000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33".to_string()),
                order_hash: SgBytes("0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4".to_string()),
                owner: SgBytes("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string()),
                outputs: vec![SgVault {
                    id: SgBytes("0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf".to_string()),
                    owner: SgBytes("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string()),
                    vault_id: SgBytes("75486334982066122983501547829219246999490818941767825330875804445439814023987".to_string()),
                    balance: SgBytes(Float::parse("0.987".to_string()).unwrap().as_hex()),
                    token: SgErc20 {
                        id: SgBytes("0x12e605bc104e93b45e1ad99f9e555f659051c2bb".to_string()),
                        address: SgBytes("0x12e605bc104e93b45e1ad99f9e555f659051c2bb".to_string()),
                        name: Some("Staked FLR".to_string()),
                        symbol: Some("sFLR".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                    orderbook: SgOrderbook {
                        id: SgBytes("0xcee8cd002f151a536394e564b84076c41bbbcd4d".to_string()),
                    },
                    orders_as_output: vec![SgOrderAsIO {
                        id: SgBytes("0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1".to_string()),
                        order_hash: SgBytes("0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4".to_string()),
                        active: true,
                    }],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                },
                SgVault {
                    id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    token: SgErc20 {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        address: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        name: Some("T1".to_string()),
                        symbol: Some("T1".to_string()),
                        decimals: Some(SgBigInt("0".to_string())),
                    },
                    balance: SgBytes(F0.as_hex()),
                    vault_id: SgBytes("0".to_string()),
                    owner: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    orders_as_output: vec![],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                    orderbook: SgOrderbook {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    }
                }],
                inputs: vec![SgVault {
                    id: SgBytes("0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac".to_string()),
                    owner: SgBytes("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string()),
                    vault_id: SgBytes("75486334982066122983501547829219246999490818941767825330875804445439814023987".to_string()),
                    balance: SgBytes(Float::parse("0.79799".to_string()).unwrap().as_hex()),
                    token: SgErc20 {
                        id: SgBytes("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d".to_string()),
                        address: SgBytes("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d".to_string()),
                        name: Some("Wrapped Flare".to_string()),
                        symbol: Some("WFLR".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                    orderbook: SgOrderbook {
                        id: SgBytes("0xcee8cd002f151a536394e564b84076c41bbbcd4d".to_string()),
                    },
                    orders_as_output: vec![],
                    orders_as_input: vec![SgOrderAsIO {
                        id: SgBytes("0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1".to_string()),
                        order_hash: SgBytes("0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4".to_string()),
                        active: true,
                    }],
                    balance_changes: vec![],
                },
                SgVault {
                    id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    token: SgErc20 {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        address: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        name: Some("T1".to_string()),
                        symbol: Some("T1".to_string()),
                        decimals: Some(SgBigInt("0".to_string())),
                    },
                    balance: SgBytes(F0.as_hex()),
                    vault_id: SgBytes("0".to_string()),
                    owner: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    orders_as_output: vec![],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                    orderbook: SgOrderbook {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    }
                }],
                orderbook: SgOrderbook {
                    id: SgBytes(CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string()),
                },
                active: true,
                timestamp_added: SgBigInt("1739448802".to_string()),
                meta: Some(SgBytes("0xff0a89c674ee7874a300590a932f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d20307846653234313143446131393344394534653833413563323334433746643332303130313838336143203078393135453336656638383239343138313633353662433337313844663836383035344638363861440a616d6f756e742d65706f6368730a74726164652d65706f6368733a63616c6c3c323e28292c0a6d61782d6f75747075743a2063616c6c3c333e28616d6f756e742d65706f6368732074726164652d65706f636873292c0a696f3a2063616c6c3c343e2874726164652d65706f636873292c0a3a63616c6c3c353e28696f293b0a0a2f2a20312e2068616e646c652d696f202a2f200a6d696e2d616d6f756e743a206d756c283120302e39292c0a3a656e7375726528677265617465722d7468616e2d6f722d657175616c2d746f286f75747075742d7661756c742d64656372656173652829206d696e2d616d6f756e742920224d696e20747261646520616d6f756e742e22292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a3a7365742868617368286f726465722d6861736828292022616d6f756e742d757365642229206164642875736564206f75747075742d7661756c742d6465637265617365282929293b0a0a2f2a20322e206765742d65706f6368202a2f200a696e697469616c2d74696d653a2063616c6c3c363e28292c0a6c6173742d74696d65205f3a2063616c6c3c373e28292c0a6475726174696f6e3a20737562286e6f77282920616e79286c6173742d74696d6520696e697469616c2d74696d652929292c0a746f74616c2d6475726174696f6e3a20737562286e6f77282920696e697469616c2d74696d65292c0a726174696f2d667265657a652d616d6f756e742d65706f6368733a2064697628312031292c0a726174696f2d667265657a652d74726164652d65706f6368733a206d756c28726174696f2d667265657a652d616d6f756e742d65706f63687320646976283630203138303029292c0a616d6f756e742d65706f6368733a2064697628746f74616c2d6475726174696f6e203630292c0a74726164652d65706f6368733a2073617475726174696e672d73756228646976286475726174696f6e20313830302920726174696f2d667265657a652d74726164652d65706f636873293b0a0a2f2a20332e20616d6f756e742d666f722d65706f6368202a2f200a616d6f756e742d65706f6368730a74726164652d65706f6368733a2c0a746f74616c2d617661696c61626c653a206c696e6561722d67726f7774682830203120616d6f756e742d65706f636873292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a756e757365643a2073756228746f74616c2d617661696c61626c652075736564292c0a64656361793a2063616c6c3c383e2874726164652d65706f636873292c0a7368792d64656361793a20657665727928677265617465722d7468616e2874726164652d65706f63687320302e303529206465636179292c0a7661726961626c652d636f6d706f6e656e743a2073756228312031292c0a7461726765742d616d6f756e743a206164642831206d756c287661726961626c652d636f6d706f6e656e74207368792d646563617929292c0a6361707065642d756e757365643a206d696e28756e75736564207461726765742d616d6f756e74293b0a0a2f2a20342e20696f2d666f722d65706f6368202a2f200a65706f63683a2c0a6c6173742d696f3a2063616c6c3c373e28292c0a6d61782d6e6578742d74726164653a20616e79286d756c286c6173742d696f20312e3031292063616c6c3c393e2829292c0a626173656c696e652d6e6578742d74726164653a206d756c286c6173742d696f2030292c0a7265616c2d626173656c696e653a206d617828626173656c696e652d6e6578742d74726164652063616c6c3c393e2829292c0a7661726961626c652d636f6d706f6e656e743a2073617475726174696e672d737562286d61782d6e6578742d7472616465207265616c2d626173656c696e65292c0a61626f76652d626173656c696e653a206d756c287661726961626c652d636f6d706f6e656e742063616c6c3c383e2865706f636829292c0a5f3a20616464287265616c2d626173656c696e652061626f76652d626173656c696e65293b0a0a2f2a20352e207365742d6c6173742d7472616465202a2f200a6c6173742d696f3a2c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229206e6f772829292c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229206c6173742d696f293b0a0a2f2a20362e206765742d696e697469616c2d74696d65202a2f200a5f3a6765742868617368286f726465722d6861736828292022696e697469616c2d74696d652229293b0a0a2f2a20372e206765742d6c6173742d7472616465202a2f200a6c6173742d74696d653a6765742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229292c0a6c6173742d696f3a6765742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229293b0a0a2f2a20382e2068616c666c696665202a2f200a65706f63683a2c0a2f2a2a0a202a20536872696e6b696e6720746865206d756c7469706c696572206c696b6520746869730a202a207468656e206170706c79696e672069742031302074696d657320616c6c6f777320666f720a202a2062657474657220707265636973696f6e207768656e206d61782d696f2d726174696f0a202a2069732076657279206c617267652c20652e672e207e31653130206f72207e316532302b0a202a0a202a205468697320776f726b7320626563617573652060706f77657260206c6f7365730a202a20707265636973696f6e206f6e20626173652060302e3560207768656e207468650a202a206578706f6e656e74206973206c6172676520616e642063616e206576656e20676f0a202a20746f20603060207768696c652074686520696f2d726174696f206973207374696c6c0a202a206c617267652e2042657474657220746f206b65657020746865206d756c7469706c6965720a202a2068696768657220707265636973696f6e20616e642064726f702074686520696f2d726174696f0a202a20736d6f6f74686c7920666f72206173206c6f6e672061732077652063616e2e0a202a0a6d756c7469706c6965723a0a2020706f77657228302e35206469762865706f636820313029292c0a76616c3a0a20206d756c280a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a2020293b0a0a2f2a20392e2073666c722d626173656c696e652d696e76202a2f200a5f3a20696e762873666c722d65786368616e67652d726174652829293b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d".to_string())),
                add_events: vec![SgAddOrder {
                    transaction: SgTransaction {
                        id: SgBytes("0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af".to_string()),
                        from: SgBytes("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string()),
                        block_number: SgBigInt("37432554".to_string()),
                        timestamp: SgBigInt("1739448802".to_string()),
                    },
                }],
                trades: vec![],
                remove_events: vec![],
            }
        }

        #[tokio::test]
        async fn test_get_orders() {
            let sg_server = MockServer::start_async().await;

            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                  "data": {
                    "orders": [
                      get_order1_json()
                    ]
                  }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                    "data": {
                      "orders": [
                        {
                          "id": "0x0000000000000000000000000000000000000000000000000000000000000234",
                          "orderBytes": "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                          "orderHash": "0x0000000000000000000000000000000000000000000000000000000000002345",
                          "owner": "0x0000000000000000000000000000000000000000",
                          "outputs": [
                            {
                              "id": "0x0000000000000000000000000000000000000000",
                              "token": {
                                "id": "0x0000000000000000000000000000000000000000",
                                "address": "0x0000000000000000000000000000000000000000",
                                "name": "T1",
                                "symbol": "T1",
                                "decimals": "0"
                              },
                              "balance": F0,
                              "vaultId": "0",
                              "owner": "0x0000000000000000000000000000000000000000",
                              "ordersAsOutput": [],
                              "ordersAsInput": [],
                              "balanceChanges": [],
                              "orderbook": {
                                "id": "0x0000000000000000000000000000000000000000"
                              }
                            }
                          ],
                          "inputs": [
                            {
                              "id": "0x0000000000000000000000000000000000000000",
                              "token": {
                                "id": "0x0000000000000000000000000000000000000000",
                                "address": "0x0000000000000000000000000000000000000000",
                                "name": "T2",
                                "symbol": "T2",
                                "decimals": "0"
                              },
                              "balance": F0,
                              "vaultId": "0",
                              "owner": "0x0000000000000000000000000000000000000000",
                              "ordersAsOutput": [],
                              "ordersAsInput": [],
                              "balanceChanges": [],
                              "orderbook": {
                                "id": "0x0000000000000000000000000000000000000000"
                              }
                            }
                          ],
                          "active": true,
                          "addEvents": [
                            {
                              "transaction": {
                                "blockNumber": "0",
                                "timestamp": "0",
                                "id": "0x0000000000000000000000000000000000000000",
                                "from": "0x0000000000000000000000000000000000000000"
                              }
                            }
                          ],
                          "meta": null,
                          "timestampAdded": "0",
                          "orderbook": {
                            "id": "0x0000000000000000000000000000000000000000"
                          },
                          "trades": [],
                          "removeEvents": []
                        }
                      ]
                    }
                  }));
            });

            let filter_args = GetOrdersFilters {
                owners: vec![],
                active: None,
                order_hash: None,
                tokens: None,
            };
            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let result = raindex_client
                .get_orders(None, Some(filter_args), Some(1))
                .await
                .unwrap();

            assert_eq!(result.len(), 2);

            let expected_order1 = RaindexOrder::try_from_sg_order(
                Rc::new(raindex_client.clone()),
                1,
                get_order1(),
                None,
            )
            .unwrap();

            let order1 = result[0].clone();
            assert_eq!(order1.chain_id, expected_order1.chain_id);
            assert_eq!(order1.id, expected_order1.id);
            assert_eq!(order1.order_bytes, expected_order1.order_bytes);
            assert_eq!(order1.order_hash, expected_order1.order_hash);
            assert_eq!(order1.owner, expected_order1.owner);
            assert_eq!(order1.outputs.len(), expected_order1.outputs.len());
            for (order1_output, expected_output) in
                order1.outputs.iter().zip(expected_order1.outputs.iter())
            {
                assert_eq!(order1_output.id(), expected_output.id());
                assert_eq!(order1_output.owner(), expected_output.owner());
                assert_eq!(order1_output.vault_id(), expected_output.vault_id());
                assert_eq!(order1_output.token().id(), expected_output.token().id());
                assert_eq!(
                    order1_output.token().address(),
                    expected_output.token().address()
                );
                assert_eq!(order1_output.token().name(), expected_output.token().name());
                assert_eq!(
                    order1_output.token().symbol(),
                    expected_output.token().symbol()
                );
                assert_eq!(
                    order1_output.token().decimals(),
                    expected_output.token().decimals()
                );
                assert_eq!(order1_output.orderbook(), expected_output.orderbook());
            }
            assert_eq!(order1.inputs.len(), expected_order1.inputs.len());
            for (order1_input, expected_input) in
                order1.inputs.iter().zip(expected_order1.inputs.iter())
            {
                assert_eq!(order1_input.id(), expected_input.id());
                assert_eq!(order1_input.owner(), expected_input.owner());
                assert_eq!(order1_input.vault_id(), expected_input.vault_id());
                assert_eq!(order1_input.token().id(), expected_input.token().id());
                assert_eq!(
                    order1_input.token().address(),
                    expected_input.token().address()
                );
                assert_eq!(order1_input.token().name(), expected_input.token().name());
                assert_eq!(
                    order1_input.token().symbol(),
                    expected_input.token().symbol()
                );
                assert_eq!(
                    order1_input.token().decimals(),
                    expected_input.token().decimals()
                );
                assert_eq!(order1_input.orderbook(), expected_input.orderbook());
            }

            assert_eq!(order1.orderbook(), expected_order1.orderbook());
            assert_eq!(order1.timestamp_added(), expected_order1.timestamp_added());

            let order2 = result[1].clone();
            assert_eq!(order2.chain_id, 137);
            assert_eq!(
                order2.id,
                B256::from_str(
                    "0x0000000000000000000000000000000000000000000000000000000000000234"
                )
                .unwrap()
            );
            assert_eq!(order2.order_bytes, Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap());
            assert_eq!(
                order2.order_hash,
                B256::from_str(
                    "0x0000000000000000000000000000000000000000000000000000000000002345"
                )
                .unwrap()
            );
            assert_eq!(
                order2.owner,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(order2.outputs.len(), 1);
            let order2_outputs = order2.outputs[0].clone();
            assert_eq!(
                order2_outputs.id(),
                Bytes::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(
                order2_outputs.owner(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(order2_outputs.vault_id(), U256::from(0));
            assert_eq!(
                order2_outputs.token().id(),
                "0x0000000000000000000000000000000000000000".to_string()
            );
            assert_eq!(
                order2_outputs.token().address(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(order2_outputs.token().name(), Some("T1".to_string()));
            assert_eq!(order2_outputs.token().symbol(), Some("T1".to_string()));
            assert_eq!(order2_outputs.token().decimals(), 0);
            assert_eq!(
                order2_outputs.orderbook(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );

            assert_eq!(order2.inputs.len(), 1);
            let order2_inputs = order2.inputs[0].clone();
            assert_eq!(
                order2_inputs.id(),
                Bytes::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(
                order2_inputs.owner(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(order2_inputs.vault_id(), U256::from(0));
            assert_eq!(
                order2_inputs.token().id(),
                "0x0000000000000000000000000000000000000000".to_string()
            );
            assert_eq!(
                order2_inputs.token().address(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(order2_inputs.token().name(), Some("T2".to_string()));
            assert_eq!(order2_inputs.token().symbol(), Some("T2".to_string()));
            assert_eq!(order2_inputs.token().decimals(), 0);
            assert_eq!(
                order2_inputs.orderbook(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(
                order2.orderbook(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(order2.timestamp_added(), U256::from(0));
        }

        #[tokio::test]
        async fn test_get_order_by_hash() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [get_order1_json()]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let res = raindex_client
                .get_order_by_hash(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
                )
                .await
                .unwrap();

            let expected_order = RaindexOrder::try_from_sg_order(
                Rc::new(raindex_client.clone()),
                1,
                get_order1(),
                None,
            )
            .unwrap();
            assert_eq!(res.id, expected_order.id);
            assert_eq!(res.order_bytes, expected_order.order_bytes);
            assert_eq!(res.order_hash, expected_order.order_hash);
            assert_eq!(res.owner, expected_order.owner);
            assert_eq!(res.outputs.len(), 2);
            assert_eq!(res.inputs.len(), 2);

            assert_eq!(res.outputs[0].id(), expected_order.outputs[0].id());
            assert_eq!(res.inputs[0].id(), expected_order.inputs[0].id());
            assert_eq!(res.inputs[1].id(), expected_order.inputs[1].id());
            assert_eq!(res.outputs[1].id(), expected_order.outputs[1].id());

            assert_eq!(res.vaults_list().items().len(), 3);
            assert_eq!(
                res.vaults_list().items()[0].id(),
                expected_order.inputs[0].id()
            );
            assert_eq!(
                res.vaults_list().items()[1].id(),
                expected_order.outputs[0].id()
            );
            assert_eq!(
                res.vaults_list().items()[2].id(),
                expected_order.inputs[1].id()
            );
        }

        #[tokio::test]
        async fn test_get_order_by_hash_not_found() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": { "orders": [] }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let order_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000123");
            let res = raindex_client
                .get_order_by_hash(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    order_hash,
                )
                .await;

            match res {
                Err(RaindexError::OrderNotFound(address, chain_id, hash)) => {
                    assert_eq!(address, CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string());
                    assert_eq!(chain_id, 1);
                    assert_eq!(hash, order_hash);
                }
                Err(err) => panic!("unexpected error {err:?}"),
                Ok(_) => panic!("expected error"),
            }
        }

        #[tokio::test]
        async fn test_invalid_meta() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [
                            json!({
                            "id": "0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1",
                            "orderBytes": "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000f08bcbce72f62c95dcb7c07dcb5ed26acfcfbc1100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000005c00000000000000000000000000000000000000000000000000000000000000640392c489ef67afdc348209452c338ea5ba2b6152b936e152f610d05e1a20621a40000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae60000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000049d000000000000000000000000000000000000000000000000000000000000000f0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000c7d713b49da0000914d696e20747261646520616d6f756e742e00000000000000000000000000008b616d6f756e742d75736564000000000000000000000000000000000000000000000000000000000000000000000000000000000000000340aad21b3b70000000000000000000000000000000000000000000000000006194049f30f7200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000e043da6172500008f6c6173742d74726164652d74696d65000000000000000000000000000000008d6c6173742d74726164652d696f0000000000000000000000000000000000008c696e697469616c2d74696d650000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b200000000000000000000000000000000000000000000000000008ac7230489e80000000000000000000000020000915e36ef882941816356bc3718df868054f868ad000000000000000000000000000000000000000000000000000000000000027d0a00000024007400e0015801b401e001f40218025c080500040b20000200100001001000000b120003001000010b110004001000030b0100051305000201100001011000003d120000011000020010000003100404211200001d02000001100003031000010c1200004911000003100404001000012b12000001100003031000010c1200004a0200001a0b00090b1000060b20000700100000001000011b1200001a10000047120000001000001a1000004712000001100000011000002e12000001100005011000042e120000001000053d12000001100004001000042e1200000010000601100005001000032e120000481200011d0b020a0010000001100000011000062713000001100003031000010c12000049110000001000030010000247120000001000010b110008001000050110000700100001201200001f12000001100000011000004712000000100006001000073d120000011000002b12000000100008001000043b120000160901080b1000070b10000901100008001000013d1200001b12000001100006001000013d1200000b100009001000033a120000001000040010000248120001001000000b110008001000053d12000000100006001000042b1200000a0401011a10000001100009031000010c1200004a020000001000000110000a031000010c1200004a020000040200010110000b031000010c120000491100000803000201100009031000010c120000491100000110000a031000010c12000049110000100c01030110000d001000002e1200000110000c3e1200000010000100100001001000010010000100100001001000010010000100100001001000013d1a0000020100010210000e3611000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33",
                            "orderHash": "0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4",
                            "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                            "outputs": [],
                            "inputs": [],
                            "orderbook": {
                                "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                            },
                            "active": true,
                            "timestampAdded": "1739448802",
                            "meta": "0x123456",
                            "addEvents": [],
                            "trades": [],
                            "removeEvents": []
                            })
                        ]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let res = raindex_client
                .get_order_by_hash(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
                )
                .await;
            assert!(res.is_ok());
        }

        #[tokio::test]
        async fn test_order_detail_extended() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [get_order1_json()]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let res = raindex_client
                .get_order_by_hash(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
                )
                .await
                .unwrap();

            assert!(res.rainlang.is_some());
            assert_eq!(res.rainlang, Some("/* 0. calculate-io */ \nusing-words-from 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC\namt: 100,\nio: call<2>();\n\n/* 1. handle-io */ \n:call<3>(),\n:ensure(equal-to(output-vault-decrease() 100) \"must take full amount\");\n\n/* 2. get-io-ratio-now */ \nelapsed: call<4>(),\nio: saturating-sub(0.0177356 div(mul(elapsed sub(0.0177356 0.0173844)) 60));\n\n/* 3. one-shot */ \n:ensure(is-zero(get(hash(order-hash() \"has-executed\"))) \"has executed\"),\n:set(hash(order-hash() \"has-executed\") 1);\n\n/* 4. get-elapsed */ \n_: sub(now() get(hash(order-hash() \"deploy-time\")));".to_string()));
        }

        #[tokio::test]
        async fn local_db_orders_parse_ios() {
            let local_orderbook = "0x0987654321098765432109876543210987654321";
            let owner = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let token = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
            let inputs_payload = serde_json::to_string(&vec![json!({
                "ioIndex": 1,
                "vault": {
                    "chainId": 137,
                    "vaultId": "0x01",
                    "token": token,
                    "owner": owner,
                    "orderbookAddress": local_orderbook,
                    "tokenName": "USDC",
                    "tokenSymbol": "USDC",
                    "tokenDecimals": 6,
                    "balance": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "inputOrders": null,
                    "outputOrders": null
                }
            })])
            .unwrap();
            let outputs_payload = serde_json::to_string(&vec![json!({
                "ioIndex": 0,
                "vault": {
                    "chainId": 137,
                    "vaultId": "0x02",
                    "token": token,
                    "owner": owner,
                    "orderbookAddress": local_orderbook,
                    "tokenName": "USDC",
                    "tokenSymbol": "USDC",
                    "tokenDecimals": 6,
                    "balance": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "inputOrders": null,
                    "outputOrders": null
                }
            })])
            .unwrap();

            let local_order = LocalDbOrder {
                chain_id: 137,
                order_hash: b256!(
                    "0x0000000000000000000000000000000000000000000000000000000000000abc"
                ),
                owner: Address::from_str(owner).unwrap(),
                block_timestamp: 1,
                block_number: 1,
                orderbook_address: Address::from_str(local_orderbook).unwrap(),
                order_bytes: Bytes::from_str("0x01").unwrap(),
                transaction_hash: b256!(
                    "0x0000000000000000000000000000000000000000000000000000000000000010"
                ),
                inputs: Some(inputs_payload),
                outputs: Some(outputs_payload),
                trade_count: 2,
                active: true,
                meta: None,
            };

            let exec = StaticJsonExec {
                json: serde_json::to_string(&vec![local_order]).unwrap(),
            };
            let local_db = LocalDb::new(exec.clone());
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "https://example/sg1",
                    "https://example/sg2",
                    "https://example/rpc1",
                    "https://example/rpc2",
                )],
                None,
            )
            .unwrap();
            client.local_db.borrow_mut().replace(local_db.clone());

            let orders_source = LocalDbOrders::new(&local_db, Rc::new(client.clone()));
            let orders = orders_source
                .list(Some(vec![137]), &GetOrdersFilters::default(), None)
                .await
                .unwrap();

            assert_eq!(orders.len(), 1);
            let order = &orders[0];
            assert_eq!(order.inputs.len(), 1);
            assert_eq!(order.outputs.len(), 1);
            assert_eq!(order.trades_count, 2);
            assert_eq!(order.orderbook, Address::from_str(local_orderbook).unwrap());
        }

        #[tokio::test]
        async fn local_db_orders_bubble_deserialization_error() {
            let exec = StaticJsonExec {
                json: "not-json".to_string(),
            };
            let local_db = LocalDb::new(exec);
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "https://example/sg1",
                    "https://example/sg2",
                    "https://example/rpc1",
                    "https://example/rpc2",
                )],
                None,
            )
            .unwrap();
            client.local_db.borrow_mut().replace(local_db.clone());

            let orders_source = LocalDbOrders::new(&local_db, Rc::new(client.clone()));
            let err = orders_source
                .list(Some(vec![137]), &GetOrdersFilters::default(), None)
                .await
                .unwrap_err();
            match err {
                RaindexError::LocalDbQueryError(LocalDbQueryError::Deserialization { .. }) => {}
                other => panic!("unexpected error: {other:?}"),
            }
        }

        #[tokio::test]
        async fn get_orders_falls_back_to_subgraph_when_no_local_db() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                  "data": {
                    "orders": [
                      {
                        "id": "0x0000000000000000000000000000000000000000000000000000000000000234",
                        "orderBytes": "0x01",
                        "orderHash": "0x0000000000000000000000000000000000000000000000000000000000002345",
                        "owner": "0x0000000000000000000000000000000000000000",
                        "outputs": [],
                        "inputs": [],
                        "active": true,
                        "addEvents": [
                          {
                            "transaction": {
                              "blockNumber": "0",
                              "timestamp": "0",
                              "id": "0x0000000000000000000000000000000000000000",
                              "from": "0x0000000000000000000000000000000000000000"
                            }
                          }
                        ],
                        "meta": null,
                        "timestampAdded": "0",
                        "orderbook": {
                          "id": "0x0987654321098765432109876543210987654321"
                        },
                        "trades": [],
                        "removeEvents": []
                      }
                    ]
                  }
                }));
            });

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "https://example/sg1",
                    &sg_server.url("/sg2"),
                    "https://example/rpc1",
                    "https://example/rpc2",
                )],
                None,
            )
            .unwrap();

            let result = client
                .get_orders(
                    Some(ChainIds(vec![137])),
                    Some(GetOrdersFilters::default()),
                    Some(1),
                )
                .await
                .unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(
                result[0].order_hash,
                b256!("0x0000000000000000000000000000000000000000000000000000000000002345")
            );
        }

        #[tokio::test]
        async fn get_order_by_hash_hits_local_before_subgraph() {
            let local_order = LocalDbOrder {
                chain_id: 137,
                order_hash: b256!(
                    "0x0000000000000000000000000000000000000000000000000000000000000abc"
                ),
                owner: Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
                block_timestamp: 1,
                block_number: 1,
                orderbook_address: Address::from_str("0x0987654321098765432109876543210987654321")
                    .unwrap(),
                order_bytes: Bytes::from_str("0x01").unwrap(),
                transaction_hash: b256!(
                    "0x0000000000000000000000000000000000000000000000000000000000000010"
                ),
                inputs: None,
                outputs: None,
                trade_count: 0,
                active: true,
                meta: None,
            };

            let exec = StaticJsonExec {
                json: serde_json::to_string(&vec![local_order]).unwrap(),
            };
            let local_db = LocalDb::new(exec);

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "https://example/sg1",
                    "https://example/sg2",
                    "https://example/rpc1",
                    "https://example/rpc2",
                )],
                None,
            )
            .unwrap();
            client.local_db.borrow_mut().replace(local_db);

            let order = client
                .get_order_by_hash(
                    &OrderbookIdentifier::new(
                        137,
                        Address::from_str("0x0987654321098765432109876543210987654321").unwrap(),
                    ),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000abc"),
                )
                .await
                .unwrap();

            assert_eq!(
                order.order_hash,
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc")
            );
            assert_eq!(order.chain_id, 137);
        }

        // TODO: Issue #1989
        // #[tokio::test]
        // async fn test_order_vaults_volume() {
        //     let sg_server = MockServer::start_async().await;
        //     sg_server.mock(|when, then| {
        //         when.path("/sg1")
        //             .body_contains("\"first\":200")
        //             .body_contains("\"skip\":0");
        //         then.status(200).json_body_obj(&json!({
        //           "data": {
        //             "trades": get_trades_json()
        //           }
        //         }));
        //     });
        //     sg_server.mock(|when, then| {
        //         when.path("/sg1")
        //             .body_contains("\"first\":200")
        //             .body_contains("\"skip\":200");
        //         then.status(200).json_body_obj(&json!({
        //             "data": { "trades": [] }
        //         }));
        //     });
        //     sg_server.mock(|when, then| {
        //         when.path("/sg1");
        //         then.status(200).json_body_obj(&json!({
        //             "data": {
        //                 "orders": [get_order1_json()]
        //             }
        //         }));
        //     });

        //     let raindex_client = RaindexClient::new(
        //         vec![get_test_yaml(
        //             &sg_server.url("/sg1"),
        //             &sg_server.url("/sg2"),
        //             // not used
        //             &sg_server.url("/rpc1"),
        //             &sg_server.url("/rpc2"),
        //         )],
        //         None,
        //     )
        //     .unwrap();
        //     let order = raindex_client
        //         .get_order_by_hash(
        //             1,
        //             Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
        //             Bytes::from_str("0x0123").unwrap(),
        //         )

        //     let volume1 = res[0].clone();
        //     assert_eq!(volume1.id(), Bytes::from_str("0x10").unwrap());
        //     assert_eq!(
        //         volume1.token().address(),
        //         Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap()
        //     );
        //     assert_eq!(volume1.token().name(), Some("Wrapped Flare".to_string()));
        //     assert_eq!(volume1.token().symbol(), Some("WFLR".to_string()));
        //     assert_eq!(volume1.token().decimals(), U256::from(18));
        //     assert_eq!(volume1.details().total_in(), U256::from(1));
        //     assert_eq!(volume1.details().total_out(), U256::from(0));
        //     assert_eq!(volume1.details().total_vol(), U256::from(1));
        //     assert_eq!(volume1.details().net_vol(), U256::from(1));

        // TODO: Issue #1989
        //     let volume1 = res[0].clone();
        //     assert_eq!(volume1.id(), U256::from_str("0x10").unwrap());
        //     assert_eq!(
        //         volume1.token().address(),
        //         Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap()
        //     );
        //     assert_eq!(volume1.token().name(), Some("Wrapped Flare".to_string()));
        //     assert_eq!(volume1.token().symbol(), Some("WFLR".to_string()));
        //     assert_eq!(volume1.token().decimals(), U256::from(18));
        //     assert_eq!(volume1.details().total_in(), U256::from(1));
        //     assert_eq!(volume1.details().total_out(), U256::from(0));
        //     assert_eq!(volume1.details().total_vol(), U256::from(1));
        //     assert_eq!(volume1.details().net_vol(), U256::from(1));

        //     let volume2 = res[1].clone();
        //     assert_eq!(volume2.id(), U256::from_str("0x10").unwrap());
        //     assert_eq!(
        //         volume2.token().address(),
        //         Address::from_str("0x12e605bc104e93b45e1ad99f9e555f659051c2bb").unwrap()
        //     );
        //     assert_eq!(volume2.token().name(), Some("Staked FLR".to_string()));
        //     assert_eq!(volume2.token().symbol(), Some("sFLR".to_string()));
        //     assert_eq!(volume2.token().decimals(), U256::from(18));
        //     assert_eq!(volume2.details().total_in(), U256::from(0));
        //     assert_eq!(volume2.details().total_out(), U256::from(2));
        //     assert_eq!(volume2.details().total_vol(), U256::from(2));
        //     assert_eq!(volume2.details().net_vol(), U256::from(2));

        //     let volume3 = res[2].clone();
        //     assert_eq!(volume3.id(), U256::from_str("0x20").unwrap());
        //     assert_eq!(
        //         volume3.token().address(),
        //         Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap()
        //     );
        //     assert_eq!(volume3.token().name(), Some("Wrapped Flare".to_string()));
        //     assert_eq!(volume3.token().symbol(), Some("WFLR".to_string()));
        //     assert_eq!(volume3.token().decimals(), U256::from(18));
        //     assert_eq!(volume3.details().total_in(), U256::from(2));
        //     assert_eq!(volume3.details().total_out(), U256::from(0));
        //     assert_eq!(volume3.details().total_vol(), U256::from(2));
        //     assert_eq!(volume3.details().net_vol(), U256::from(2));

        //     let volume4 = res[3].clone();
        //     assert_eq!(volume4.id(), U256::from_str("0x20").unwrap());
        //     assert_eq!(
        //         volume4.token().address(),
        //         Address::from_str("0x12e605bc104e93b45e1ad99f9e555f659051c2bb").unwrap()
        //     );
        //     assert_eq!(volume4.token().name(), Some("Staked FLR".to_string()));
        //     assert_eq!(volume4.token().symbol(), Some("sFLR".to_string()));
        //     assert_eq!(volume4.token().decimals(), U256::from(18));
        //     assert_eq!(volume4.details().total_in(), U256::from(0));
        //     assert_eq!(volume4.details().total_out(), U256::from(5));
        //     assert_eq!(volume4.details().total_vol(), U256::from(5));
        //     assert_eq!(volume4.details().net_vol(), U256::from(5));
        // }

        //     let raindex_client = RaindexClient::new(
        //         vec![get_test_yaml(
        //             &sg_server.url("/sg1"),
        //             &sg_server.url("/sg2"),
        //             // not used
        //             &sg_server.url("/rpc1"),
        //             &sg_server.url("/rpc2"),
        //         )],
        //         None,
        //     )
        //     .unwrap();
        //     let order = raindex_client
        //         .get_order_by_hash(
        //             1,
        //             Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
        //             Bytes::from_str("0x0123").unwrap(),
        //         )
        //         .await
        //         .unwrap();
        //     let res = order
        //         .get_performance(Some(1632000000), Some(1734571449))
        //         .await
        //         .unwrap();

        //     assert_eq!(res.order_id, "order1");
        //     assert_eq!(res.order_hash, "0x1");
        //     assert_eq!(res.orderbook, "0x0000000000000000000000000000000000000000");
        //     assert_eq!(
        //         res.denominated_performance,
        //         Some(DenominatedPerformance {
        //             token: SgErc20 {
        //                 id: SgBytes("token-2".to_string()),
        //                 address: SgBytes("0x2222222222222222222222222222222222222222".to_string()),
        //                 name: Some("Token Two".to_string()),
        //                 symbol: Some("TK2".to_string()),
        //                 decimals: Some(SgBigInt("18".to_string())),
        //             },
        //             apy: U256::from(0),
        //             apy_is_neg: false,
        //             net_vol: U256::from(0),
        //             net_vol_is_neg: false,
        //             starting_capital: U256::from(600),
        //         })
        //     );
        //     assert_eq!(res.start_time, 1632000000);
        //     assert_eq!(res.end_time, 1734571449);
        //     assert_eq!(res.inputs_vaults.len(), 1);
        //     assert_eq!(
        //         res.inputs_vaults[0],
        //         VaultPerformance {
        //             id: "2".to_string(),
        //             token: SgErc20 {
        //                 id: SgBytes("token-2".to_string()),
        //                 address: SgBytes("0x2222222222222222222222222222222222222222".to_string()),
        //                 name: Some("Token Two".to_string()),
        //                 symbol: Some("TK2".to_string()),
        //                 decimals: Some(SgBigInt("18".to_string())),
        //             },
        //             vol_details: VolumeDetails {
        //                 total_in: U256::from(50000000000000000000u128),
        //                 total_out: U256::from(0u8),
        //                 total_vol: U256::from(50000000000000000000u128),
        //                 net_vol: U256::from(50000000000000000000u128),
        //             },
        //             apy_details: Some(APYDetails {
        //                 start_time: 1632000000,
        //                 end_time: 1734571449,
        //                 net_vol: U256::from(50000000000000000000u128),
        //                 capital: U256::from(150u8),
        //                 apy: Some(U256::from(102484659254448087225972733172491493u128)),
        //                 is_neg: false,
        //             }),
        //         }
        //     );
        //     assert_eq!(res.outputs_vaults.len(), 1);
        //     assert_eq!(
        //         res.outputs_vaults[0],
        //         VaultPerformance {
        //             id: "1".to_string(),
        //             token: SgErc20 {
        //                 id: SgBytes("token-1".to_string()),
        //                 address: SgBytes("0x1111111111111111111111111111111111111111".to_string()),
        //                 name: Some("Token One".to_string()),
        //                 symbol: Some("TK1".to_string()),
        //                 decimals: Some(SgBigInt("18".to_string())),
        //             },
        //             vol_details: VolumeDetails {
        //                 total_in: U256::from(0),
        //                 total_out: U256::from(100000000000000000000u128),
        //                 total_vol: U256::from(100000000000000000000u128),
        //                 net_vol: U256::from(100000000000000000000u128),
        //             },
        //             apy_details: Some(APYDetails {
        //                 start_time: 1632000000,
        //                 end_time: 1734571449,
        //                 net_vol: U256::from(100000000000000000000u128),
        //                 capital: U256::from(900u16),
        //                 apy: Some(U256::from(34161553084816029075324244390830497u128)),
        //                 is_neg: true,
        //             }),
        //         }
        //     );
        // }

        // TODO: Issue #1989
        // #[tokio::test]
        // async fn test_get_orders_with_token_filter() {
        //     let sg_server = MockServer::start_async().await;

        //     sg_server.mock(|when, then| {
        //         when.path("/sg1")
        //             .body_contains("\"token_in\":[\"0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\"]");
        //         then.status(200).json_body_obj(&json!({
        //             "data": {
        //                 "orders": [get_order1_json()]
        //             }
        //         }));
        //     });
        //     sg_server.mock(|when, then| {
        //         when.path("/sg2")
        //             .body_contains("\"token_in\":[\"0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\"]");
        //         then.status(200).json_body_obj(&json!({
        //             "data": {
        //                 "orders": []
        //             }
        //         }));
        //     });

        //     let raindex_client = RaindexClient::new(
        //         vec![get_test_yaml(
        //             &sg_server.url("/sg1"),
        //             &sg_server.url("/sg2"),
        //             &sg_server.url("/rpc1"),
        //             &sg_server.url("/rpc2"),
        //         )],
        //         None,
        //     )
        //     .unwrap();

        //     let filters = GetOrdersFilters {
        //         owners: vec![],
        //         active: None,
        //         order_hash: None,
        //         tokens: Some(vec![Address::from_str(
        //             "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
        //         )
        //         .unwrap()]),
        //     };

        //     let result = raindex_client
        //         .get_orders(None, Some(filters), None)
        //         .await
        //         .unwrap();

        //     assert_eq!(result.len(), 1);
        //     assert_eq!(
        //         result[0].id,
        //         Bytes::from_str(
        //             "0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1"
        //         )
        //         .unwrap()
        //     );
        // }

        // TODO: Issue #1989
        // #[tokio::test]
        // async fn test_get_orders_with_multiple_token_filters() {
        //     let sg_server = MockServer::start_async().await;

        //     sg_server.mock(|when, then| {
        //         when.path("/sg1")
        //             .body_contains("\"token_in\":[\"0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\",\"0x12e605bc104e93b45e1ad99f9e555f659051c2bb\"]");
        //         then.status(200).json_body_obj(&json!({
        //             "data": {
        //                 "orders": [get_order1_json()]
        //             }
        //         }));
        //     });
        //     sg_server.mock(|when, then| {
        //         when.path("/sg2");
        //         then.status(200).json_body_obj(&json!({
        //             "data": {
        //                 "orders": []
        //             }
        //         }));
        //     });

        //     let raindex_client = RaindexClient::new(
        //         vec![get_test_yaml(
        //             &sg_server.url("/sg1"),
        //             &sg_server.url("/sg2"),
        //             &sg_server.url("/rpc1"),
        //             &sg_server.url("/rpc2"),
        //         )],
        //         None,
        //     )
        //     .unwrap();

        //     let filters = GetOrdersFilters {
        //         owners: vec![],
        //         active: None,
        //         order_hash: None,
        //         tokens: Some(vec![
        //             Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap(),
        //             Address::from_str("0x12e605bc104e93b45e1ad99f9e555f659051c2bb").unwrap(),
        //         ]),
        //     };

        //     let result = raindex_client
        //         .get_orders(None, Some(filters), None)
        //         .await
        //         .unwrap();

        //     assert_eq!(result.len(), 1);
        // }
    }
}
