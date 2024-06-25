use crate::schema;
use serde::Serialize;
use typeshare::typeshare;
#[derive(cynic::QueryVariables, Debug, Clone)]
#[typeshare]
pub struct OrderTakesListQueryVariables {
    pub first: Option<i32>,
    pub id: Bytes,
    pub skip: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "OrderTakesListQueryVariables")]
#[typeshare]
pub struct OrderTakesListQuery {
    #[arguments(skip: $skip, first: $first, where: { order_: { id: $id } })]
    pub trades: Vec<Trade>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Trade {
    pub id: Bytes,
    pub trade_event: TradeEvent,
    pub output_vault_balance_change: TradeVaultBalanceChange,
    pub order: Order,
    pub input_vault_balance_change: TradeVaultBalanceChange2,
    pub timestamp: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "TradeVaultBalanceChange")]
#[typeshare]
pub struct TradeVaultBalanceChange2 {
    pub vault: Vault,
    pub amount: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct TradeVaultBalanceChange {
    pub amount: BigInt,
    pub vault: Vault,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Vault {
    pub token: Bytes,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct TradeEvent {
    pub transaction: Transaction,
    pub sender: Bytes,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Transaction {
    pub id: Bytes,
    pub from: Bytes,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Order {
    pub id: Bytes,
    pub order_hash: Bytes,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[typeshare]
pub enum OrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "Trade_orderBy")]
#[typeshare]
pub enum TradeOrderBy {
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
    #[cynic(rename = "tradeEvent")]
    TradeEvent,
    #[cynic(rename = "tradeEvent__id")]
    TradeEventId,
    #[cynic(rename = "tradeEvent__sender")]
    TradeEventSender,
    #[cynic(rename = "order__meta")]
    Meta,
    #[cynic(rename = "order__timestampAdded")]
    OrderTimestampAdded,
    #[cynic(rename = "inputVaultBalanceChange__timestamp")]
    InputVaultBalanceChangeTimestamp,
    #[cynic(rename = "outputVaultBalanceChange__timestamp")]
    OutputVaultBalanceChangeTimestamp,
    #[cynic(rename = "timestamp")]
    Timestamp,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]
pub struct Bytes(pub String);
