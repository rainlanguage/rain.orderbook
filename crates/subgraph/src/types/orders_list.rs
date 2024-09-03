use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug, Clone)]
#[typeshare]
pub struct OrdersListQueryVariables {
    pub first: Option<i32>,
    pub skip: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "OrdersListQueryVariables")]
#[typeshare]
pub struct OrdersListQuery {
    #[arguments(orderBy: lastModified, orderDirection: "desc", skip: $skip, first: $first)]
    pub orders: Vec<Order>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Order {
    pub id: Bytes,
    pub order_bytes: Bytes,
    pub order_hash: Bytes,
    pub owner: Bytes,
    pub outputs: Vec<Vault>,
    pub inputs: Vec<Vault>,
    pub active: bool,
    pub add_events: Vec<AddOrder>,
    pub timestamp_added: BigInt,
    pub orderbook: Orderbook,
    pub last_modified: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Orderbook {
    pub id: Bytes,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Vault {
    pub token: ERC20,
    pub balance: BigInt,
    pub vault_id: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct ERC20 {
    pub id: Bytes,
    pub address: Bytes,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<BigInt>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct AddOrder {
    pub transaction: Transaction,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Transaction {
    pub id: Bytes,
    pub block_number: BigInt,
    pub timestamp: BigInt,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "AddOrder_orderBy")]
#[typeshare]
pub enum AddOrderOrderBy {
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
    #[cynic(rename = "order__meta")]
    Meta,
    #[cynic(rename = "order__timestampAdded")]
    OrderTimestampAdded,
    #[cynic(rename = "order__lastModified")]
    OrderLastModified,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[typeshare]
pub enum OrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]
pub struct Bytes(pub String);
