use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug, Clone)]
#[typeshare]
pub struct VaultsListQueryVariables {
    pub first: Option<i32>,
    pub skip: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "VaultsListQueryVariables")]
#[typeshare]
pub struct VaultsListQuery {
    #[arguments(orderBy: "id", orderDirection: "desc", skip: $skip, first: $first)]
    pub vaults: Vec<Vault>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Vault {
    pub id: Bytes,
    pub vault_id: BigInt,
    pub owner: Bytes,
    pub token: ERC20,
    pub balance: BigInt,
    pub orders_as_input: Vec<Order>,
    pub orders_as_output: Vec<Order>,
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
pub struct Order {
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
#[cynic(graphql_type = "Vault_orderBy")]
#[typeshare]
pub enum VaultOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "token")]
    Token,
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
    #[cynic(rename = "token__address")]
    TokenAddress,
    #[cynic(rename = "token__decimals")]
    TokenDecimals,
    #[cynic(rename = "token__id")]
    TokenId,
    #[cynic(rename = "token__name")]
    TokenName,
    #[cynic(rename = "token__symbol")]
    TokenSymbol,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]

pub struct Bytes(pub String);
