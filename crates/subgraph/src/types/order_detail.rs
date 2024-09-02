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

#[derive(cynic::QueryVariables, Debug, Clone)]
#[typeshare]
pub struct BatchOrderDetailQueryVariables {
    #[cynic(rename = "id_list")]
    pub id_list: OrderIdList,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "Order_filter")]
#[typeshare]
pub struct OrderIdList {
    #[cynic(rename = "id_in")]
    pub id_in: Vec<Bytes>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "BatchOrderDetailQueryVariables")]
#[typeshare]
pub struct BatchOrderDetailQuery {
    #[arguments(where: $id_list)]
    pub orders: Vec<Order>,
}

#[typeshare]
pub type RainMetaV1 = Bytes;

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
    pub meta: Option<RainMetaV1>,
    pub timestamp_added: BigInt,
    pub orderbook: Orderbook,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Orderbook {
    pub id: Bytes,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Vault {
    pub id: Bytes,
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
