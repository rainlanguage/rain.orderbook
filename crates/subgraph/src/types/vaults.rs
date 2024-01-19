use crate::schema;

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct VaultsQuery {
    #[arguments(orderBy: "owner__id", orderDirection: "desc")]
    pub vaults: Vec<Vault>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct Vault {
    pub id: cynic::Id,
    pub owner: Account,
    pub token_vaults: Option<Vec<TokenVault>>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct TokenVault {
    pub id: cynic::Id,
    pub balance_display: BigDecimal,
    pub balance: BigInt,
    pub token: Erc20,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub total_supply: BigInt,
    pub total_supply_display: BigDecimal,
}

#[typeshare]
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

#[typeshare]
#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "Vault_orderBy")]
pub enum VaultOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "vaultId")]
    VaultId,
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

#[typeshare]
#[serde(rename = "BigDecimalString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[typeshare]
#[serde(rename = "BigIntString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[typeshare]
#[serde(rename = "BytesString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
