use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug, Clone)]
#[typeshare]
pub struct OrderDetailQueryVariables {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "OrderDetailQueryVariables")]
#[typeshare]
pub struct OrderDetailQuery {
    #[arguments(id: $id)]
    pub order: Option<Order>,
}

#[typeshare]
pub type RainMetaV1 = Bytes;

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Order {
    pub order_bytes: Bytes,
    pub order_hash: Bytes,
    pub owner: Bytes,
    pub outputs: Vec<Vault>,
    pub inputs: Vec<Vault>,
    pub active: bool,
    #[arguments(first: 1, orderBy: "transaction__timestamp", orderDirection: "desc")]
    pub add_events: Vec<AddOrder>,
    pub meta: Option<RainMetaV1>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Vault {
    pub token: Bytes,
    pub balance: BigInt,
    pub vault_id: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct AddOrder {
    pub transaction: Transaction,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Transaction {
    pub block_number: BigInt,
    pub timestamp: BigInt,
}

#[derive(cynic::Enum, Copy, Debug, Clone)]
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
    Timestamp,
}

#[derive(cynic::Enum, Copy, Debug, Clone)]
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
