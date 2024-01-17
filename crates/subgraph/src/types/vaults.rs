#[cynic::schema("orderbook")]
pub mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct VaultsQuery {
    #[arguments(orderBy: "owner__id", orderDirection: "desc")]
    pub vaults: Vec<Vault>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Vault {
    pub id: cynic::Id,
    pub owner: Account,
    pub token_vaults: Option<Vec<TokenVault>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct TokenVault {
    pub id: cynic::Id,
    pub token: Erc20,
    pub balance_display: BigDecimal,
    pub balance: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
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
#[cynic(graphql_type = "Vault_orderBy")]
pub enum VaultOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "owner")]
    Owner,
    #[cynic(rename = "owner__id")]
    OwnerId,
    #[cynic(rename = "tokenVaults")]
    TokenVaults,
    #[cynic(rename = "deposits")]
    Deposits,
    #[cynic(rename = "withdraws")]
    Withdraws,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
