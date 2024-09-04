use super::common::*;
use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
#[typeshare]
pub struct BatchOrderDetailQueryVariables {
    #[cynic(rename = "id_list")]
    pub id_list: OrderIdList,
}

// #[derive(cynic::QueryVariables, Debug)]
// #[typeshare]
// pub struct OrdersListQueryVariables {
//     pub first: Option<i32>,
//     pub skip: Option<i32>,
// }

// #[derive(cynic::QueryVariables, Debug)]
// #[typeshare]
// pub struct OrderDetailQueryVariables<'a> {
//     pub id: &'a cynic::Id,
// }

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

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "PaginationQueryVariables")]
#[typeshare]
pub struct OrdersListQuery {
    #[arguments(orderBy: "timestampAdded", orderDirection: "desc", skip: $skip, first: $first)]
    pub orders: Vec<Order>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "IdQueryVariables")]
#[typeshare]
pub struct OrderDetailQuery {
    #[arguments(id: $id)]
    pub order: Option<Order>,
}

// #[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
// #[typeshare]
// pub struct Order {
//     pub id: Bytes,
//     pub order_bytes: Bytes,
//     pub order_hash: Bytes,
//     pub owner: Bytes,
//     pub outputs: Vec<Vault>,
//     pub inputs: Vec<Vault>,
//     pub orderbook: Orderbook,
//     pub active: bool,
//     pub timestamp_added: BigInt,
//     pub meta: Option<Bytes>,
//     #[arguments(first: 1, orderBy: "transaction__timestamp", orderDirection: "desc")]
//     pub add_events: Vec<AddOrder>,
// }

// #[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
// #[typeshare]
// pub struct Orderbook {
//     pub id: Bytes,
// }

// #[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
// #[typeshare]
// pub struct Vault {
//     pub token: Erc20,
//     pub balance: BigInt,
//     pub vault_id: BigInt,
// }

// #[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
// #[cynic(graphql_type = "ERC20")]
// #[typeshare]
// pub struct Erc20 {
//     pub id: Bytes,
//     pub address: Bytes,
//     pub name: Option<String>,
//     pub symbol: Option<String>,
//     pub decimals: Option<BigInt>,
// }

// #[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
// #[typeshare]
// pub struct AddOrder {
//     pub transaction: Transaction,
// }

// #[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
// #[typeshare]
// pub struct Transaction {
//     pub id: Bytes,
//     pub block_number: BigInt,
//     pub timestamp: BigInt,
// }

// #[derive(cynic::Enum, Clone, Copy, Debug)]
// #[cynic(graphql_type = "AddOrder_orderBy")]
// #[typeshare]
// pub enum AddOrderOrderBy {
//     #[cynic(rename = "id")]
//     Id,
//     #[cynic(rename = "order")]
//     Order,
//     #[cynic(rename = "order__id")]
//     OrderId,
//     #[cynic(rename = "order__active")]
//     OrderActive,
//     #[cynic(rename = "order__orderHash")]
//     OrderOrderHash,
//     #[cynic(rename = "order__owner")]
//     OrderOwner,
//     #[cynic(rename = "order__nonce")]
//     OrderNonce,
//     #[cynic(rename = "order__orderBytes")]
//     OrderOrderBytes,
//     #[cynic(rename = "order__meta")]
//     OrderMeta,
//     #[cynic(rename = "order__timestampAdded")]
//     OrderTimestampAdded,
//     #[cynic(rename = "orderbook")]
//     Orderbook,
//     #[cynic(rename = "orderbook__id")]
//     OrderbookId,
//     #[cynic(rename = "transaction")]
//     Transaction,
//     #[cynic(rename = "transaction__id")]
//     TransactionId,
//     #[cynic(rename = "transaction__timestamp")]
//     TransactionTimestamp,
//     #[cynic(rename = "transaction__blockNumber")]
//     TransactionBlockNumber,
//     #[cynic(rename = "transaction__from")]
//     TransactionFrom,
//     #[cynic(rename = "sender")]
//     Sender,
// }

// #[derive(cynic::Enum, Clone, Copy, Debug)]
// #[typeshare]
// pub enum OrderDirection {
//     #[cynic(rename = "asc")]
//     Asc,
//     #[cynic(rename = "desc")]
//     Desc,
// }

// #[derive(cynic::Enum, Clone, Copy, Debug)]
// #[cynic(graphql_type = "Order_orderBy")]
// #[typeshare]
// pub enum OrderOrderBy {
//     #[cynic(rename = "id")]
//     Id,
//     #[cynic(rename = "orderbook")]
//     Orderbook,
//     #[cynic(rename = "orderbook__id")]
//     OrderbookId,
//     #[cynic(rename = "active")]
//     Active,
//     #[cynic(rename = "orderHash")]
//     OrderHash,
//     #[cynic(rename = "owner")]
//     Owner,
//     #[cynic(rename = "inputs")]
//     Inputs,
//     #[cynic(rename = "outputs")]
//     Outputs,
//     #[cynic(rename = "nonce")]
//     Nonce,
//     #[cynic(rename = "orderBytes")]
//     OrderBytes,
//     #[cynic(rename = "addEvents")]
//     AddEvents,
//     #[cynic(rename = "removeEvents")]
//     RemoveEvents,
//     #[cynic(rename = "trades")]
//     Trades,
//     #[cynic(rename = "meta")]
//     Meta,
//     #[cynic(rename = "timestampAdded")]
//     TimestampAdded,
// }

// #[derive(cynic::Scalar, Debug, Clone)]
// #[typeshare]
// pub struct BigInt(pub String);

// #[derive(cynic::Scalar, Debug, Clone)]
// #[typeshare]
// pub struct Bytes(pub String);
