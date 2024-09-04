use super::common::*;
use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

// #[derive(cynic::QueryVariables, Debug)]
// #[typeshare]
// pub struct VaultDetailQueryVariables<'a> {
//     pub id: &'a cynic::Id,
// }

// #[derive(cynic::QueryVariables, Debug)]
// #[typeshare]
// pub struct VaultsListQueryVariables {
//     pub first: Option<i32>,
//     pub skip: Option<i32>,
// }

// #[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
// #[typeshare]
// pub struct Withdrawal {
//     pub id: Bytes,
//     pub __typename: String,
//     pub amount: BigInt,
//     pub old_vault_balance: BigInt,
//     pub new_vault_balance: BigInt,
// }

// #[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
// #[typeshare]
// pub struct TradeVaultBalanceChange {
//     pub id: Bytes,
//     pub __typename: String,
//     pub amount: BigInt,
//     pub old_vault_balance: BigInt,
//     pub new_vault_balance: BigInt,
// }

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "PaginationQueryVariables")]
#[typeshare]
pub struct VaultsListQuery {
    #[arguments(orderBy: "id", orderDirection: "desc", skip: $skip, first: $first)]
    pub vaults: Vec<Vault>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "IdQueryVariables")]
#[typeshare]
pub struct VaultDetailQuery {
    #[arguments(id: $id)]
    pub vault: Option<Vault>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "PaginationWithIdQueryVariables")]
#[typeshare]
pub struct VaultBalanceChangesListQuery {
    #[arguments(orderDirection: "desc", orderBy: "timestamp", where: { vault_: { id: $id } }, skip: $skip, first: $first)]
    pub vault_balance_changes: Vec<VaultBalanceChangeUnwrapped>,
}

// #[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
// #[typeshare]
// pub struct Vault {
//     pub id: Bytes,
//     pub owner: Bytes,
//     pub vault_id: BigInt,
//     pub balance: BigInt,
//     pub token: Erc20,
//     pub orderbook: Orderbook,
//     pub orders_as_output: Vec<Order>,
//     pub orders_as_input: Vec<Order>,
//     pub balance_changes: Vec<VaultBalanceChange>,
// }

// #[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
// #[typeshare]
// pub struct Orderbook {
//     pub id: Bytes,
// }

// #[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
// #[typeshare]
// pub struct Order {
//     pub id: Bytes,
//     pub order_hash: Bytes,
//     pub active: bool,
// }

// #[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
// #[cynic(graphql_type = "ERC20")]
// #[typeshare]
// pub struct Erc20 {
//     pub id: Bytes,
//     pub address: Bytes,
//     pub name: Option<String>,
//     pub symbol: Option<String>,
//     pub decimals: Option<BigInt>,
// }

// #[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
// #[typeshare]
// pub struct Deposit {
//     pub id: Bytes,
//     pub __typename: String,
//     pub amount: BigInt,
//     pub old_vault_balance: BigInt,
//     pub new_vault_balance: BigInt,
// }

// #[derive(cynic::InlineFragments, Debug, Clone, Serialize)]
// #[typeshare]
// pub enum VaultBalanceChange {
//     Withdrawal(Withdrawal),
//     TradeVaultBalanceChange(TradeVaultBalanceChange),
//     Deposit(Deposit),
//     #[cynic(fallback)]
//     Unknown,
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
// #[cynic(graphql_type = "Vault_orderBy")]
// #[typeshare]
// pub enum VaultOrderBy {
//     #[cynic(rename = "id")]
//     Id,
//     #[cynic(rename = "orderbook")]
//     Orderbook,
//     #[cynic(rename = "orderbook__id")]
//     OrderbookId,
//     #[cynic(rename = "token")]
//     Token,
//     #[cynic(rename = "token__id")]
//     TokenId,
//     #[cynic(rename = "token__address")]
//     TokenAddress,
//     #[cynic(rename = "token__name")]
//     TokenName,
//     #[cynic(rename = "token__symbol")]
//     TokenSymbol,
//     #[cynic(rename = "token__decimals")]
//     TokenDecimals,
//     #[cynic(rename = "owner")]
//     Owner,
//     #[cynic(rename = "vaultId")]
//     VaultId,
//     #[cynic(rename = "ordersAsInput")]
//     OrdersAsInput,
//     #[cynic(rename = "ordersAsOutput")]
//     OrdersAsOutput,
//     #[cynic(rename = "balance")]
//     Balance,
//     #[cynic(rename = "balanceChanges")]
//     BalanceChanges,
// }

// #[derive(cynic::Scalar, Debug, Clone)]
// #[typeshare]
// pub struct BigInt(pub String);

// #[derive(cynic::Scalar, Debug, Clone)]
// #[typeshare]
// pub struct Bytes(pub String);
