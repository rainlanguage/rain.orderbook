use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct VaultsQuery {
    #[arguments(orderBy: "owner__id", orderDirection: "desc")]
    pub token_vaults: Vec<TokenVault>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct TokenVault {
    pub id: cynic::Id,
    pub owner: Account,
    pub vault_id: BigInt,
    pub token: Erc20,
    pub balance_display: BigDecimal,
    pub balance: BigInt,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub id: cynic::Id,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Account {
    pub id: Bytes,
}

#[typeshare]
#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum OrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

#[typeshare]
#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "TokenVault_orderBy")]
pub enum TokenVaultOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "owner")]
    Owner,
    #[cynic(rename = "owner__id")]
    OwnerId,
    #[cynic(rename = "vault")]
    Vault,
    #[cynic(rename = "vault__id")]
    VaultId,
    #[cynic(rename = "vault__vaultId")]
    VaultVaultId,
    #[cynic(rename = "vaultId")]
    VaultId2,
    #[cynic(rename = "token")]
    Token,
    #[cynic(rename = "token__id")]
    TokenId,
    #[cynic(rename = "token__name")]
    TokenName,
    #[cynic(rename = "token__symbol")]
    TokenSymbol,
    #[cynic(rename = "token__totalSupply")]
    TokenTotalSupply,
    #[cynic(rename = "token__totalSupplyDisplay")]
    TokenTotalSupplyDisplay,
    #[cynic(rename = "token__decimals")]
    TokenDecimals,
    #[cynic(rename = "balance")]
    Balance,
    #[cynic(rename = "balanceDisplay")]
    BalanceDisplay,
    #[cynic(rename = "orders")]
    Orders,
    #[cynic(rename = "orderClears")]
    OrderClears,
    #[cynic(rename = "takeOrders")]
    TakeOrders,
}

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
