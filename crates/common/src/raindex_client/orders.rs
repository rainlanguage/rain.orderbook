use super::*;
use crate::{
    meta::TryDecodeRainlangSource,
    raindex_client::{
        transactions::RaindexTransaction,
        vaults::{RaindexVault, RaindexVaultType},
    },
};
use alloy::primitives::{Address, Bytes, U256};
use rain_orderbook_subgraph_client::{
    performance::{vol::VaultVolume, OrderPerformance},
    types::{
        common::{
            SgBigInt, SgBytes, SgOrder, SgOrderAsIO, SgOrderbook, SgOrdersListFilterArgs, SgVault,
        },
        Id,
    },
    MultiOrderbookSubgraphClient, OrderbookSubgraphClient, SgPaginationArgs,
};
use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, RwLock, RwLockReadGuard},
};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::BigInt;

const DEFAULT_PAGE_SIZE: u16 = 100;

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
    raindex_client: Arc<RwLock<RaindexClient>>,
    chain_id: u32,
    id: Bytes,
    order_bytes: Bytes,
    order_hash: Bytes,
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
    #[wasm_bindgen(getter)]
    pub fn inputs(&self) -> Vec<RaindexVault> {
        self.inputs.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn outputs(&self) -> Vec<RaindexVault> {
        self.outputs.clone()
    }
    /// Returns a combined view of all vaults associated with this order.
    ///
    /// This method merges input and output vaults, properly handling vaults that serve
    /// both roles by marking them as InputOutput type. The returned list contains each
    /// unique vault exactly once with the correct type classification.
    ///
    /// ## Returns
    ///
    /// - `Vec<RaindexVault>` - All vaults with proper type classification
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// order.vaults.forEach(vault => {
    ///   console.log(`${vault.id}: ${vault.vaultType}`);
    /// });
    /// ```
    #[wasm_bindgen(getter)]
    pub fn vaults(&self) -> Vec<RaindexVault> {
        get_vaults_with_type(self.inputs.clone(), self.outputs.clone())
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
}
#[cfg(not(target_family = "wasm"))]
impl RaindexOrder {
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    pub fn id(&self) -> Bytes {
        self.id.clone()
    }
    pub fn order_bytes(&self) -> Bytes {
        self.order_bytes.clone()
    }
    pub fn order_hash(&self) -> Bytes {
        self.order_hash.clone()
    }
    pub fn owner(&self) -> Address {
        self.owner
    }
    pub fn inputs(&self) -> Vec<RaindexVault> {
        self.inputs.clone()
    }
    pub fn outputs(&self) -> Vec<RaindexVault> {
        self.outputs.clone()
    }
    pub fn vaults(&self) -> Vec<RaindexVault> {
        get_vaults_with_type(self.inputs.clone(), self.outputs.clone())
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
    pub fn get_raindex_client(&self) -> Result<RwLockReadGuard<RaindexClient>, RaindexError> {
        self.raindex_client
            .read()
            .map_err(|_| RaindexError::ReadLockError)
    }
    #[wasm_export(skip)]
    pub fn get_orderbook_client(&self) -> Result<OrderbookSubgraphClient, RaindexError> {
        let raindex_client = self.get_raindex_client()?;
        raindex_client.get_orderbook_client(self.chain_id, self.orderbook)
    }

    #[wasm_export(skip)]
    pub fn get_rpc_urls(&self) -> Result<Vec<Url>, RaindexError> {
        let raindex_client = self.get_raindex_client()?;
        raindex_client.get_rpc_urls_for_chain(self.chain_id)
    }

    /// Retrieves volume data for all vaults associated with this order over a specified time period
    ///
    /// Queries historical volume information across all vaults that belong to this order,
    /// allowing analysis of trading activity and liquidity patterns over time.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await order.getVaultsVolume(
    ///   Math.floor(Date.now() / 1000) - 86400, // 24 hours ago
    ///   Math.floor(Date.now() / 1000)
    /// );
    /// if (result.error) {
    ///   console.error("Error fetching volume:", result.error.readableMsg);
    ///   return;
    /// }
    /// const volumes = result.value;
    /// // Do something with volumes
    /// ```
    #[wasm_export(
        js_name = "getVaultsVolume",
        return_description = "Volume data for each vault over the specified period",
        unchecked_return_type = "VaultVolume[]",
        preserve_js_class
    )]
    pub async fn get_vaults_volume(
        &self,
        #[wasm_export(
            js_name = "startTimestamp",
            param_description = "Unix timestamp for the start of the query period (optional)"
        )]
        start_timestamp: Option<u64>,
        #[wasm_export(
            js_name = "endTimestamp",
            param_description = "Unix timestamp for the end of the query period (optional)"
        )]
        end_timestamp: Option<u64>,
    ) -> Result<Vec<VaultVolume>, RaindexError> {
        let client = self.get_orderbook_client()?;
        let volumes = client
            .order_vaults_volume(Id::new(self.id.to_string()), start_timestamp, end_timestamp)
            .await?;
        Ok(volumes)
    }

    /// Gets comprehensive performance metrics and analytics for this order over a specified time period
    ///
    /// Retrieves detailed performance data including profit/loss, volume statistics, and other
    /// key metrics that help assess the effectiveness of the trading strategy implemented by this order.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await order.getPerformance(
    ///   Math.floor(Date.now() / 1000) - 604800, // 1 week ago
    ///   Math.floor(Date.now() / 1000)
    /// );
    /// if (result.error) {
    ///   console.error("Error fetching performance:", result.error.readableMsg);
    ///   return;
    /// }
    /// const performance = result.value;
    /// // Do something with performance
    /// ```
    #[wasm_export(
        js_name = "getPerformance",
        return_description = "Comprehensive performance metrics for the order",
        unchecked_return_type = "OrderPerformance"
    )]
    pub async fn get_performance(
        &self,
        #[wasm_export(
            js_name = "startTimestamp",
            param_description = "Unix timestamp for the start of the analysis period (optional, defaults to order creation)"
        )]
        start_timestamp: Option<u64>,
        #[wasm_export(
            js_name = "endTimestamp",
            param_description = "Unix timestamp for the end of the analysis period (optional, defaults to current time)"
        )]
        end_timestamp: Option<u64>,
    ) -> Result<OrderPerformance, RaindexError> {
        let client = self.get_orderbook_client()?;
        let performance = client
            .order_performance(Id::new(self.id.to_string()), start_timestamp, end_timestamp)
            .await?;
        Ok(performance)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RaindexOrderAsIO {
    #[tsify(type = "Hex")]
    pub id: Bytes,
    #[tsify(type = "Hex")]
    pub order_hash: Bytes,
    pub active: bool,
}
impl_wasm_traits!(RaindexOrderAsIO);
impl TryFrom<SgOrderAsIO> for RaindexOrderAsIO {
    type Error = RaindexError;
    fn try_from(order: SgOrderAsIO) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Bytes::from_str(&order.id.0)?,
            order_hash: Bytes::from_str(&order.order_hash.0)?,
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
        return_description = "Array of orders matching the specified criteria",
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
        let raindex_client = Arc::new(RwLock::new(self.clone()));
        let multi_subgraph_args =
            self.get_multi_subgraph_args(chain_ids.map(|ids| ids.0.to_vec()))?;

        let client = MultiOrderbookSubgraphClient::new(
            multi_subgraph_args.values().flatten().cloned().collect(),
        );

        let orders = client
            .orders_list(
                filters
                    .unwrap_or(GetOrdersFilters {
                        owners: vec![],
                        active: None,
                        order_hash: None,
                    })
                    .try_into()?,
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
                let order = RaindexOrder::try_from_sg_order(
                    raindex_client.clone(),
                    chain_id,
                    order.order.clone(),
                    None,
                )?;
                Ok(order)
            })
            .collect::<Result<Vec<RaindexOrder>, RaindexError>>()?;
        Ok(orders)
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
        let order_hash = Bytes::from_str(&order_hash)?;
        self._get_order_by_hash(chain_id, orderbook_address, order_hash)
            .await
    }
}
impl RaindexClient {
    async fn _get_order_by_hash(
        &self,
        chain_id: u32,
        orderbook_address: Address,
        order_hash: Bytes,
    ) -> Result<RaindexOrder, RaindexError> {
        let raindex_client = Arc::new(RwLock::new(self.clone()));
        let client = self.get_orderbook_client(chain_id, orderbook_address)?;
        let order = client
            .order_detail_by_hash(SgBytes(order_hash.to_string()))
            .await?;
        let order = RaindexOrder::try_from_sg_order(raindex_client.clone(), chain_id, order, None)?;
        Ok(order)
    }
    pub async fn get_order_by_hash(
        &self,
        chain_id: u32,
        orderbook_address: Address,
        order_hash: Bytes,
    ) -> Result<RaindexOrder, RaindexError> {
        self._get_order_by_hash(chain_id, orderbook_address, order_hash)
            .await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct GetOrdersFilters {
    #[tsify(type = "Address[]")]
    pub owners: Vec<Address>,
    #[tsify(optional)]
    pub active: Option<bool>,
    #[tsify(optional, type = "Hex")]
    pub order_hash: Option<Bytes>,
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
        })
    }
}

impl RaindexOrder {
    pub fn try_from_sg_order(
        raindex_client: Arc<RwLock<RaindexClient>>,
        chain_id: u32,
        order: SgOrder,
        transaction: Option<RaindexTransaction>,
    ) -> Result<Self, RaindexError> {
        let rainlang = order
            .meta
            .as_ref()
            .map(|meta| meta.0.try_decode_rainlangsource())
            .transpose()?;
        Ok(Self {
            raindex_client: raindex_client.clone(),
            chain_id,
            id: Bytes::from_str(&order.id.0)?,
            order_bytes: Bytes::from_str(&order.order_bytes.0)?,
            order_hash: Bytes::from_str(&order.order_hash.0)?,
            owner: Address::from_str(&order.owner.0)?,
            inputs: order
                .inputs
                .iter()
                .map(|v| {
                    RaindexVault::try_from_sg_vault(
                        raindex_client.clone(),
                        chain_id,
                        v.clone(),
                        Some(RaindexVaultType::Input),
                    )
                })
                .collect::<Result<Vec<RaindexVault>, RaindexError>>()?,
            outputs: order
                .outputs
                .iter()
                .map(|v| {
                    RaindexVault::try_from_sg_vault(
                        raindex_client.clone(),
                        chain_id,
                        v.clone(),
                        Some(RaindexVaultType::Output),
                    )
                })
                .collect::<Result<Vec<RaindexVault>, RaindexError>>()?,
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
                .outputs()
                .into_iter()
                .map(|v| v.into_sg_vault())
                .collect::<Result<Vec<SgVault>, RaindexError>>()?,
            inputs: self
                .inputs()
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
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use alloy::primitives::U256;
        use httpmock::MockServer;
        use rain_orderbook_subgraph_client::{
            performance::{
                apy::APYDetails, vol::VolumeDetails, DenominatedPerformance, VaultPerformance,
            },
            types::common::{
                SgAddOrder, SgBigInt, SgBytes, SgErc20, SgOrderAsIO, SgOrderbook, SgTransaction,
                SgVault,
            },
        };
        use serde_json::{json, Value};

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
                  "balance": "987000000000000000",
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
                    "balance": "0",
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
                  "balance": "797990000000000000",
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
                    "balance": "0",
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
                vault_id: SgBigInt("75486334982066122983501547829219246999490818941767825330875804445439814023987".to_string()),
                balance: SgBigInt("987000000000000000".to_string()),
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
                balance: SgBigInt("0".to_string()),
                vault_id: SgBigInt("0".to_string()),
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
                vault_id: SgBigInt("75486334982066122983501547829219246999490818941767825330875804445439814023987".to_string()),
                balance: SgBigInt("797990000000000000".to_string()),
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
                balance: SgBigInt("0".to_string()),
                vault_id: SgBigInt("0".to_string()),
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
        fn get_trades_json() -> Value {
            json!([
              {
                "id": "trade1",
                "tradeEvent": {
                  "transaction": {
                    "id": "tx1",
                    "from": "from1",
                    "blockNumber": "0",
                    "timestamp": "0"
                  },
                  "sender": "sender1"
                },
                "outputVaultBalanceChange": {
                  "id": "ovbc1",
                  "__typename": "TradeVaultBalanceChange",
                  "amount": "-2",
                  "newVaultBalance": "0",
                  "oldVaultBalance": "0",
                  "vault": {
                    "id": "vault1",
                    "vaultId": "1",
                    "token": {
                      "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                      "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                      "name": "Staked FLR",
                      "symbol": "sFLR",
                      "decimals": "18"
                    }
                  },
                  "timestamp": "1700000000",
                  "transaction": {
                    "id": "tx1",
                    "from": "from1",
                    "blockNumber": "0",
                    "timestamp": "1700000000"
                  },
                  "orderbook": {
                    "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                  }
                },
                "order": {
                  "id": "order1",
                  "orderHash": "hash1"
                },
                "inputVaultBalanceChange": {
                  "id": "ivbc1",
                  "__typename": "TradeVaultBalanceChange",
                  "amount": "1",
                  "newVaultBalance": "0",
                  "oldVaultBalance": "0",
                  "vault": {
                    "id": "vault1",
                    "vaultId": "1",
                    "token": {
                      "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                      "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                      "name": "Wrapped Flare",
                      "symbol": "WFLR",
                      "decimals": "18"
                    }
                  },
                  "timestamp": "1700000000",
                  "transaction": {
                    "id": "tx1",
                    "from": "from1",
                    "blockNumber": "0",
                    "timestamp": "1700000000"
                  },
                  "orderbook": {
                    "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                  }
                },
                "timestamp": "0",
                "orderbook": {
                  "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                }
              },
              {
                "id": "trade2",
                "tradeEvent": {
                  "transaction": {
                    "id": "tx2",
                    "from": "from2",
                    "blockNumber": "0",
                    "timestamp": "0"
                  },
                  "sender": "sender2"
                },
                "outputVaultBalanceChange": {
                  "id": "ovbc2",
                  "__typename": "TradeVaultBalanceChange",
                  "amount": "-5",
                  "newVaultBalance": "0",
                  "oldVaultBalance": "0",
                  "vault": {
                    "id": "vault2",
                    "vaultId": "2",
                    "token": {
                      "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                      "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                      "name": "Staked FLR",
                      "symbol": "sFLR",
                      "decimals": "18"
                    }
                  },
                  "timestamp": "1700086400",
                  "transaction": {
                    "id": "tx2",
                    "from": "from2",
                    "blockNumber": "0",
                    "timestamp": "1700086400"
                  },
                  "orderbook": {
                    "id": "ob2"
                  }
                },
                "order": {
                  "id": "order2",
                  "orderHash": "hash2"
                },
                "inputVaultBalanceChange": {
                  "id": "ivbc2",
                  "__typename": "TradeVaultBalanceChange",
                  "amount": "2",
                  "newVaultBalance": "0",
                  "oldVaultBalance": "0",
                  "vault": {
                    "id": "vault2",
                    "vaultId": "2",
                    "token": {
                      "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                      "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                      "name": "Wrapped Flare",
                      "symbol": "WFLR",
                      "decimals": "18"
                    }
                  },
                  "timestamp": "0",
                  "transaction": {
                    "id": "tx2",
                    "from": "from2",
                    "blockNumber": "0",
                    "timestamp": "1700086400"
                  },
                  "orderbook": {
                    "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                  }
                },
                "timestamp": "1700086400",
                "orderbook": {
                  "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                }
              }
            ])
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
                          "id": "0x0234",
                          "orderBytes": "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                          "orderHash": "0x2345",
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
                              "balance": "0",
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
                              "balance": "0",
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
                Arc::new(RwLock::new(raindex_client.clone())),
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
                    order1_output.token().decimals().unwrap(),
                    expected_output.token().decimals().unwrap()
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
                    order1_input.token().decimals().unwrap(),
                    expected_input.token().decimals().unwrap()
                );
                assert_eq!(order1_input.orderbook(), expected_input.orderbook());
            }

            assert_eq!(order1.orderbook(), expected_order1.orderbook());
            assert_eq!(order1.timestamp_added(), expected_order1.timestamp_added());

            let order2 = result[1].clone();
            assert_eq!(order2.chain_id, 137);
            assert_eq!(order2.id, Bytes::from_str("0x0234").unwrap());
            assert_eq!(order2.order_bytes, Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap());
            assert_eq!(order2.order_hash, Bytes::from_str("0x2345").unwrap());
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
            assert_eq!(order2_outputs.token().decimals(), Some(U256::from(0)));
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
            assert_eq!(order2_inputs.token().decimals(), Some(U256::from(0)));
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
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();

            let expected_order = RaindexOrder::try_from_sg_order(
                Arc::new(RwLock::new(raindex_client.clone())),
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

            assert_eq!(res.vaults().len(), 3);
            assert_eq!(res.vaults()[0].id(), expected_order.inputs[0].id());
            assert_eq!(res.vaults()[1].id(), expected_order.outputs[0].id());
            assert_eq!(res.vaults()[2].id(), expected_order.inputs[1].id());
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
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();

            assert!(res.rainlang.is_some());
            assert_eq!(res.rainlang, Some("/* 0. calculate-io */ \nusing-words-from 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC\namt: 100,\nio: call<2>();\n\n/* 1. handle-io */ \n:call<3>(),\n:ensure(equal-to(output-vault-decrease() 100) \"must take full amount\");\n\n/* 2. get-io-ratio-now */ \nelapsed: call<4>(),\nio: saturating-sub(0.0177356 div(mul(elapsed sub(0.0177356 0.0173844)) 60));\n\n/* 3. one-shot */ \n:ensure(is-zero(get(hash(order-hash() \"has-executed\"))) \"has executed\"),\n:set(hash(order-hash() \"has-executed\") 1);\n\n/* 4. get-elapsed */ \n_: sub(now() get(hash(order-hash() \"deploy-time\")));".to_string()));
        }

        #[tokio::test]
        async fn test_order_vaults_volume() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":0");
                then.status(200).json_body_obj(&json!({
                  "data": {
                    "trades": get_trades_json()
                  }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":200");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [] }
                }));
            });
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
            let order = raindex_client
                .get_order_by_hash(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let res = order.get_vaults_volume(None, None).await.unwrap();

            assert_eq!(res.len(), 4);

            let volume1 = res[0].clone();
            assert_eq!(volume1.id, "1");
            assert_eq!(
                volume1.token.address.0,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(volume1.token.name, Some("Wrapped Flare".to_string()));
            assert_eq!(volume1.token.symbol, Some("WFLR".to_string()));
            assert_eq!(volume1.token.decimals, Some(SgBigInt("18".to_string())));
            assert_eq!(
                volume1.vol_details,
                VolumeDetails {
                    total_in: U256::from(1),
                    total_out: U256::from(0),
                    total_vol: U256::from(1),
                    net_vol: U256::from(1),
                }
            );

            let volume2 = res[1].clone();
            assert_eq!(volume2.id, "1");
            assert_eq!(
                volume2.token.address.0,
                "0x12e605bc104e93b45e1ad99f9e555f659051c2bb"
            );
            assert_eq!(volume2.token.name, Some("Staked FLR".to_string()));
            assert_eq!(volume2.token.symbol, Some("sFLR".to_string()));
            assert_eq!(volume2.token.decimals, Some(SgBigInt("18".to_string())));
            assert_eq!(
                volume2.vol_details,
                VolumeDetails {
                    total_in: U256::from(0),
                    total_out: U256::from(2),
                    total_vol: U256::from(2),
                    net_vol: U256::from(2),
                }
            );

            let volume3 = res[2].clone();
            assert_eq!(volume3.id, "2");
            assert_eq!(
                volume3.token.address.0,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(volume3.token.name, Some("Wrapped Flare".to_string()));
            assert_eq!(volume3.token.symbol, Some("WFLR".to_string()));
            assert_eq!(volume3.token.decimals, Some(SgBigInt("18".to_string())));
            assert_eq!(
                volume3.vol_details,
                VolumeDetails {
                    total_in: U256::from(2),
                    total_out: U256::from(0),
                    total_vol: U256::from(2),
                    net_vol: U256::from(2),
                }
            );

            let volume4 = res[3].clone();
            assert_eq!(volume4.id, "2");
            assert_eq!(
                volume4.token.address.0,
                "0x12e605bc104e93b45e1ad99f9e555f659051c2bb"
            );
            assert_eq!(volume4.token.name, Some("Staked FLR".to_string()));
            assert_eq!(volume4.token.symbol, Some("sFLR".to_string()));
            assert_eq!(volume4.token.decimals, Some(SgBigInt("18".to_string())));
            assert_eq!(
                volume4.vol_details,
                VolumeDetails {
                    total_in: U256::from(0),
                    total_out: U256::from(5),
                    total_vol: U256::from(5),
                    net_vol: U256::from(5),
                }
            );
        }

        #[tokio::test]
        async fn test_order_performance() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1").body_contains("SgOrderDetailByIdQuery");
                then.status(200).json_body_obj(&json!({
                  "data": {
                    "order": {
                      "id": "order1",
                      "orderBytes": "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                      "orderHash": "0x1",
                      "owner": "0x0000000000000000000000000000000000000000",
                      "outputs": [
                        {
                          "id": "0x0000000000000000000000000000000000000000",
                          "token": {
                            "id": "token-1",
                            "address": "0x1111111111111111111111111111111111111111",
                            "name": "Token One",
                            "symbol": "TK1",
                            "decimals": "18"
                          },
                          "balance": "0",
                          "vaultId": "1",
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
                            "id": "token-2",
                            "address": "0x2222222222222222222222222222222222222222",
                            "name": "Token Two",
                            "symbol": "TK2",
                            "decimals": "18"
                          },
                          "balance": "0",
                          "vaultId": "2",
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
                  }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":0");
                then.status(200).json_body_obj(&json!({
                  "data": {
                    "trades": [
                      {
                        "id": "0x07db8b3f3e7498f9d4d0e40b98f57c020d3d277516e86023a8200a20464d4894",
                        "timestamp": "1632000000",
                        "tradeEvent": {
                          "sender": "0x0000000000000000000000000000000000000000",
                          "transaction": {
                            "id": "0x0000000000000000000000000000000000000000",
                            "from": "0x0000000000000000000000000000000000000000",
                            "timestamp": "1632000000",
                            "blockNumber": "0"
                          }
                        },
                        "outputVaultBalanceChange": {
                          "amount": "-100000000000000000000",
                          "vault": {
                            "id": "vault-1",
                            "vaultId": "1",
                            "token": {
                              "id": "token-1",
                              "address": "0x1111111111111111111111111111111111111111",
                              "name": "Token One",
                              "symbol": "TK1",
                              "decimals": "18"
                            }
                          },
                          "id": "output-change-1",
                          "__typename": "TradeVaultBalanceChange",
                          "newVaultBalance": "900",
                          "oldVaultBalance": "1000",
                          "timestamp": "1632000000",
                          "transaction": {
                            "id": "0x0000000000000000000000000000000000000000",
                            "from": "0x0000000000000000000000000000000000000000",
                            "timestamp": "1632000000",
                            "blockNumber": "0"
                          },
                          "orderbook": {
                            "id": "orderbook-1"
                          }
                        },
                        "order": {
                          "id": "order1.id",
                          "orderHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                        },
                        "inputVaultBalanceChange": {
                          "amount": "50000000000000000000",
                          "vault": {
                            "id": "vault-2",
                            "vaultId": "2",
                            "token": {
                              "id": "token-2",
                              "address": "0x2222222222222222222222222222222222222222",
                              "name": "Token Two",
                              "symbol": "TK2",
                              "decimals": "18"
                            }
                          },
                          "id": "input-change-1",
                          "__typename": "TradeVaultBalanceChange",
                          "newVaultBalance": "150",
                          "oldVaultBalance": "100",
                          "timestamp": "1632000000",
                          "transaction": {
                            "id": "0x0000000000000000000000000000000000000000",
                            "from": "0x0000000000000000000000000000000000000000",
                            "timestamp": "1632000000",
                            "blockNumber": "0"
                          },
                          "orderbook": {
                            "id": "orderbook-1"
                          }
                        },
                        "orderbook": {
                          "id": "orderbook-1"
                        }
                      }
                    ]
                  }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":200");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [] }
                }));
            });
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
            let order = raindex_client
                .get_order_by_hash(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let res = order
                .get_performance(Some(1632000000), Some(1734571449))
                .await
                .unwrap();

            assert_eq!(res.order_id, "order1");
            assert_eq!(res.order_hash, "0x1");
            assert_eq!(res.orderbook, "0x0000000000000000000000000000000000000000");
            assert_eq!(
                res.denominated_performance,
                Some(DenominatedPerformance {
                    token: SgErc20 {
                        id: SgBytes("token-2".to_string()),
                        address: SgBytes("0x2222222222222222222222222222222222222222".to_string()),
                        name: Some("Token Two".to_string()),
                        symbol: Some("TK2".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                    apy: U256::from(0),
                    apy_is_neg: false,
                    net_vol: U256::from(0),
                    net_vol_is_neg: false,
                    starting_capital: U256::from(600),
                })
            );
            assert_eq!(res.start_time, 1632000000);
            assert_eq!(res.end_time, 1734571449);
            assert_eq!(res.inputs_vaults.len(), 1);
            assert_eq!(
                res.inputs_vaults[0],
                VaultPerformance {
                    id: "2".to_string(),
                    token: SgErc20 {
                        id: SgBytes("token-2".to_string()),
                        address: SgBytes("0x2222222222222222222222222222222222222222".to_string()),
                        name: Some("Token Two".to_string()),
                        symbol: Some("TK2".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                    vol_details: VolumeDetails {
                        total_in: U256::from(50000000000000000000u128),
                        total_out: U256::from(0u8),
                        total_vol: U256::from(50000000000000000000u128),
                        net_vol: U256::from(50000000000000000000u128),
                    },
                    apy_details: Some(APYDetails {
                        start_time: 1632000000,
                        end_time: 1734571449,
                        net_vol: U256::from(50000000000000000000u128),
                        capital: U256::from(150u8),
                        apy: Some(U256::from(102484659254448087225972733172491493u128)),
                        is_neg: false,
                    }),
                }
            );
            assert_eq!(res.outputs_vaults.len(), 1);
            assert_eq!(
                res.outputs_vaults[0],
                VaultPerformance {
                    id: "1".to_string(),
                    token: SgErc20 {
                        id: SgBytes("token-1".to_string()),
                        address: SgBytes("0x1111111111111111111111111111111111111111".to_string()),
                        name: Some("Token One".to_string()),
                        symbol: Some("TK1".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                    vol_details: VolumeDetails {
                        total_in: U256::from(0),
                        total_out: U256::from(100000000000000000000u128),
                        total_vol: U256::from(100000000000000000000u128),
                        net_vol: U256::from(100000000000000000000u128),
                    },
                    apy_details: Some(APYDetails {
                        start_time: 1632000000,
                        end_time: 1734571449,
                        net_vol: U256::from(100000000000000000000u128),
                        capital: U256::from(900u16),
                        apy: Some(U256::from(34161553084816029075324244390830497u128)),
                        is_neg: true,
                    }),
                }
            );
        }
    }
}
