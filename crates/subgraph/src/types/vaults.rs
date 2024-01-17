#[cynic::schema("orderbook")]
pub mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct VaultsQuery {
    #[arguments(orderBy: "owner__id", orderDirection: "desc")]
    pub token_vaults: Vec<TokenVault>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TokenVault {
    pub id: cynic::Id,
    pub owner: Account,
    pub token: Erc20,
    pub balance: BigInt,
    pub balance_display: BigDecimal,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub total_supply: BigInt,
    pub total_supply_display: BigDecimal,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub id: Bytes,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum OrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

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
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
