use crate::schema;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(cynic::QueryVariables, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgIdQueryVariables<'a> {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub id: &'a cynic::Id,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SgOrdersListFilterArgs {
    pub owners: Vec<SgBytes>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub active: Option<bool>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub order_hash: Option<SgBytes>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgPaginationQueryVariables {
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub first: Option<i32>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub skip: Option<i32>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "Order_filter")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgOrdersListQueryFilters {
    #[cynic(rename = "owner_in", skip_serializing_if = "Vec::is_empty")]
    pub owner_in: Vec<SgBytes>,
    #[cynic(rename = "active", skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[cynic(rename = "orderHash", skip_serializing_if = "Option::is_none")]
    pub order_hash: Option<SgBytes>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgOrdersListQueryVariables {
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub first: Option<i32>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub skip: Option<i32>,
    #[cynic(rename = "filters")]
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub filters: Option<SgOrdersListQueryFilters>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgPaginationWithIdQueryVariables {
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub first: Option<i32>,
    pub id: SgBytes,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub skip: Option<i32>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgPaginationWithTimestampQueryVariables {
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub first: Option<i32>,
    pub id: SgBytes,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub skip: Option<i32>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub timestamp_gte: Option<SgBigInt>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub timestamp_lte: Option<SgBigInt>,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "Orderbook")]
pub struct SgOrderbook {
    pub id: SgBytes,
}

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type SgRainMetaV1 = SgBytes;

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "Order")]
pub struct SgOrder {
    pub id: SgBytes,
    pub order_bytes: SgBytes,
    pub order_hash: SgBytes,
    pub owner: SgBytes,
    pub outputs: Vec<SgVault>,
    pub inputs: Vec<SgVault>,
    pub orderbook: SgOrderbook,
    pub active: bool,
    pub timestamp_added: SgBigInt,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub meta: Option<SgRainMetaV1>,
    pub add_events: Vec<SgAddOrder>,
    pub trades: Vec<SgOrderStructPartialTrade>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SgOrderWithSubgraphName {
    pub order: SgOrder,
    pub subgraph_name: String,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "Order")]
#[serde(rename_all = "camelCase")]
pub struct SgTradeStructPartialOrder {
    pub id: SgBytes,
    pub order_hash: SgBytes,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "Order")]
#[serde(rename_all = "camelCase")]
pub struct SgOrderAsIO {
    pub id: SgBytes,
    pub order_hash: SgBytes,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SgVaultsListFilterArgs {
    pub owners: Vec<SgBytes>,
    pub hide_zero_balance: bool,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "Vault_filter")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgVaultsListQueryFilters {
    #[cynic(rename = "owner_in", skip_serializing_if = "Vec::is_empty")]
    pub owner_in: Vec<SgBytes>,
    #[cynic(rename = "balance_gt", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub balance_gt: Option<SgBigInt>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgVaultsListQueryVariables {
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub first: Option<i32>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub skip: Option<i32>,
    #[cynic(rename = "filters")]
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub filters: Option<SgVaultsListQueryFilters>,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "Vault")]
pub struct SgVault {
    pub id: SgBytes,
    pub owner: SgBytes,
    pub vault_id: SgBigInt,
    pub balance: SgBigInt,
    pub token: SgErc20,
    pub orderbook: SgOrderbook,
    // latest orders
    #[arguments(orderBy: timestampAdded, orderDirection: desc)]
    pub orders_as_output: Vec<SgOrderAsIO>,
    // latest orders
    #[arguments(orderBy: timestampAdded, orderDirection: desc)]
    pub orders_as_input: Vec<SgOrderAsIO>,
    pub balance_changes: Vec<SgVaultBalanceChangeType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SgVaultWithSubgraphName {
    pub vault: SgVault,
    pub subgraph_name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "Vault")]
#[serde(rename_all = "camelCase")]
pub struct SgVaultBalanceChangeVault {
    pub id: SgBytes,
    pub vault_id: SgBigInt,
    pub token: SgErc20,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "VaultBalanceChange")]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgVaultBalanceChangeUnwrapped {
    #[serde(rename = "__typename")]
    pub __typename: String,
    pub amount: SgBigInt,
    pub new_vault_balance: SgBigInt,
    pub old_vault_balance: SgBigInt,
    pub vault: SgVaultBalanceChangeVault,
    pub timestamp: SgBigInt,
    pub transaction: SgTransaction,
    pub orderbook: SgOrderbook,
}

#[derive(cynic::InlineFragments, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(tag = "__typename", content = "data")]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "VaultBalanceChange")]
pub enum SgVaultBalanceChangeType {
    Withdrawal(SgWithdrawal),
    TradeVaultBalanceChange(SgTradeVaultBalanceChange),
    Deposit(SgDeposit),
    ClearBounty(SgClearBounty),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "Deposit")]
pub struct SgDeposit {
    pub id: SgBytes,
    #[serde(rename = "__typename")]
    pub __typename: String,
    pub amount: SgBigInt,
    pub new_vault_balance: SgBigInt,
    pub old_vault_balance: SgBigInt,
    pub vault: SgVaultBalanceChangeVault,
    pub timestamp: SgBigInt,
    pub transaction: SgTransaction,
    pub orderbook: SgOrderbook,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "Withdrawal")]
pub struct SgWithdrawal {
    pub id: SgBytes,
    #[serde(rename = "__typename")]
    pub __typename: String,
    pub amount: SgBigInt,
    pub new_vault_balance: SgBigInt,
    pub old_vault_balance: SgBigInt,
    pub vault: SgVaultBalanceChangeVault,
    pub timestamp: SgBigInt,
    pub transaction: SgTransaction,
    pub orderbook: SgOrderbook,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "TradeVaultBalanceChange")]
pub struct SgTradeVaultBalanceChange {
    pub id: SgBytes,
    #[serde(rename = "__typename")]
    pub __typename: String,
    pub amount: SgBigInt,
    pub new_vault_balance: SgBigInt,
    pub old_vault_balance: SgBigInt,
    pub vault: SgVaultBalanceChangeVault,
    pub timestamp: SgBigInt,
    pub transaction: SgTransaction,
    pub orderbook: SgOrderbook,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "ClearBounty")]
pub struct SgClearBounty {
    pub id: SgBytes,
    #[serde(rename = "__typename")]
    pub __typename: String,
    pub amount: SgBigInt,
    pub new_vault_balance: SgBigInt,
    pub old_vault_balance: SgBigInt,
    pub vault: SgVaultBalanceChangeVault,
    pub timestamp: SgBigInt,
    pub transaction: SgTransaction,
    pub orderbook: SgOrderbook,
    pub sender: SgBytes,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "TradeEvent")]
pub struct SgTradeEvent {
    pub transaction: SgTransaction,
    pub sender: SgBytes,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "Trade")]
pub struct SgTrade {
    pub id: SgBytes,
    pub trade_event: SgTradeEvent,
    pub output_vault_balance_change: SgTradeVaultBalanceChange,
    pub order: SgTradeStructPartialOrder,
    pub input_vault_balance_change: SgTradeVaultBalanceChange,
    pub timestamp: SgBigInt,
    pub orderbook: SgOrderbook,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "Trade")]
pub struct SgOrderStructPartialTrade {
    pub id: SgBytes,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "ERC20")]
pub struct SgErc20 {
    pub id: SgBytes,
    pub address: SgBytes,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub name: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub symbol: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub decimals: Option<SgBigInt>,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[cynic(graphql_type = "Transaction")]
pub struct SgTransaction {
    pub id: SgBytes,
    pub from: SgBytes,
    pub block_number: SgBigInt,
    pub timestamp: SgBigInt,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "AddOrder")]
pub struct SgAddOrder {
    pub transaction: SgTransaction,
}

#[derive(cynic::Scalar, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "BigInt")]
pub struct SgBigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[cynic(graphql_type = "Bytes")]
pub struct SgBytes(pub String);

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cfg_attr(target_family = "wasm", derive(Tsify), tsify(namespace))]
#[cynic(graphql_type = "OrderDirection")]
pub enum SgOrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "Order_orderBy")]
#[cfg_attr(target_family = "wasm", derive(Tsify), tsify(namespace))]
pub enum SgOrderOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "orderbook")]
    Orderbook,
    #[cynic(rename = "orderbook__id")]
    OrderbookId,
    #[cynic(rename = "active")]
    Active,
    #[cynic(rename = "orderHash")]
    OrderHash,
    #[cynic(rename = "owner")]
    Owner,
    #[cynic(rename = "inputs")]
    Inputs,
    #[cynic(rename = "outputs")]
    Outputs,
    #[cynic(rename = "nonce")]
    Nonce,
    #[cynic(rename = "orderBytes")]
    OrderBytes,
    #[cynic(rename = "addEvents")]
    AddEvents,
    #[cynic(rename = "removeEvents")]
    RemoveEvents,
    #[cynic(rename = "trades")]
    Trades,
    #[cynic(rename = "meta")]
    Meta,
    #[cynic(rename = "timestampAdded")]
    TimestampAdded,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "AddOrder_orderBy")]
#[cfg_attr(target_family = "wasm", derive(Tsify), tsify(namespace))]
pub enum SgAddOrderOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "order")]
    Order,
    #[cynic(rename = "order__id")]
    OrderId,
    #[cynic(rename = "order__active")]
    OrderActive,
    #[cynic(rename = "order__orderHash")]
    OrderOrderHash,
    #[cynic(rename = "order__owner")]
    OrderOwner,
    #[cynic(rename = "order__nonce")]
    OrderNonce,
    #[cynic(rename = "order__orderBytes")]
    OrderOrderBytes,
    #[cynic(rename = "order__meta")]
    OrderMeta,
    #[cynic(rename = "order__timestampAdded")]
    OrderTimestampAdded,
    #[cynic(rename = "orderbook")]
    Orderbook,
    #[cynic(rename = "orderbook__id")]
    OrderbookId,
    #[cynic(rename = "transaction")]
    Transaction,
    #[cynic(rename = "transaction__id")]
    TransactionId,
    #[cynic(rename = "transaction__timestamp")]
    TransactionTimestamp,
    #[cynic(rename = "transaction__blockNumber")]
    TransactionBlockNumber,
    #[cynic(rename = "transaction__from")]
    TransactionFrom,
    #[cynic(rename = "sender")]
    Sender,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "Trade_orderBy")]
#[cfg_attr(target_family = "wasm", derive(Tsify), tsify(namespace))]
pub enum SgTradeOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "orderbook")]
    Orderbook,
    #[cynic(rename = "orderbook__id")]
    OrderbookId,
    #[cynic(rename = "order")]
    Order,
    #[cynic(rename = "order__id")]
    OrderId,
    #[cynic(rename = "order__active")]
    OrderActive,
    #[cynic(rename = "order__orderHash")]
    OrderOrderHash,
    #[cynic(rename = "order__owner")]
    OrderOwner,
    #[cynic(rename = "order__nonce")]
    OrderNonce,
    #[cynic(rename = "order__orderBytes")]
    OrderOrderBytes,
    #[cynic(rename = "order__meta")]
    OrderMeta,
    #[cynic(rename = "order__timestampAdded")]
    OrderTimestampAdded,
    #[cynic(rename = "inputVaultBalanceChange")]
    InputVaultBalanceChange,
    #[cynic(rename = "inputVaultBalanceChange__id")]
    InputVaultBalanceChangeId,
    #[cynic(rename = "inputVaultBalanceChange__amount")]
    InputVaultBalanceChangeAmount,
    #[cynic(rename = "inputVaultBalanceChange__oldVaultBalance")]
    InputVaultBalanceChangeOldVaultBalance,
    #[cynic(rename = "inputVaultBalanceChange__newVaultBalance")]
    InputVaultBalanceChangeNewVaultBalance,
    #[cynic(rename = "inputVaultBalanceChange__timestamp")]
    InputVaultBalanceChangeTimestamp,
    #[cynic(rename = "outputVaultBalanceChange")]
    OutputVaultBalanceChange,
    #[cynic(rename = "outputVaultBalanceChange__id")]
    OutputVaultBalanceChangeId,
    #[cynic(rename = "outputVaultBalanceChange__amount")]
    OutputVaultBalanceChangeAmount,
    #[cynic(rename = "outputVaultBalanceChange__oldVaultBalance")]
    OutputVaultBalanceChangeOldVaultBalance,
    #[cynic(rename = "outputVaultBalanceChange__newVaultBalance")]
    OutputVaultBalanceChangeNewVaultBalance,
    #[cynic(rename = "outputVaultBalanceChange__timestamp")]
    OutputVaultBalanceChangeTimestamp,
    #[cynic(rename = "tradeEvent")]
    TradeEvent,
    #[cynic(rename = "tradeEvent__id")]
    TradeEventId,
    #[cynic(rename = "tradeEvent__sender")]
    TradeEventSender,
    #[cynic(rename = "timestamp")]
    Timestamp,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "Vault_orderBy")]
#[cfg_attr(target_family = "wasm", derive(Tsify), tsify(namespace))]
pub enum SgVaultOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "orderbook")]
    Orderbook,
    #[cynic(rename = "orderbook__id")]
    OrderbookId,
    #[cynic(rename = "token")]
    Token,
    #[cynic(rename = "token__id")]
    TokenId,
    #[cynic(rename = "token__address")]
    TokenAddress,
    #[cynic(rename = "token__name")]
    TokenName,
    #[cynic(rename = "token__symbol")]
    TokenSymbol,
    #[cynic(rename = "token__decimals")]
    TokenDecimals,
    #[cynic(rename = "owner")]
    Owner,
    #[cynic(rename = "vaultId")]
    VaultId,
    #[cynic(rename = "ordersAsInput")]
    OrdersAsInput,
    #[cynic(rename = "ordersAsOutput")]
    OrdersAsOutput,
    #[cynic(rename = "balance")]
    Balance,
    #[cynic(rename = "balanceChanges")]
    BalanceChanges,
}

#[cfg(target_family = "wasm")]
mod impls {
    use super::*;

    impl_wasm_traits!(SgOrder);
    impl_wasm_traits!(SgVault);
    impl_wasm_traits!(SgAddOrder);
    impl_wasm_traits!(SgOrderAsIO);
    impl_wasm_traits!(SgVaultBalanceChangeVault);
    impl_wasm_traits!(SgVaultBalanceChangeType);
    impl_wasm_traits!(SgWithdrawal);
    impl_wasm_traits!(SgTradeVaultBalanceChange);
    impl_wasm_traits!(SgDeposit);
    impl_wasm_traits!(SgClearBounty);
    impl_wasm_traits!(SgOrderStructPartialTrade);
    impl_wasm_traits!(SgErc20);
    impl_wasm_traits!(SgTransaction);
    impl_wasm_traits!(SgBigInt);
    impl_wasm_traits!(SgBytes);
    impl_wasm_traits!(SgOrdersListFilterArgs);
    impl_wasm_traits!(SgVaultsListFilterArgs);
    impl_wasm_traits!(SgTrade);
    impl_wasm_traits!(SgTradeStructPartialOrder);
    impl_wasm_traits!(SgTradeEvent);
}
