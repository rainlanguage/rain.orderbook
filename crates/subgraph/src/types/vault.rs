use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryVariables, Debug)]
pub struct VaultQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "Query", variables = "VaultQueryVariables")]
pub struct VaultQuery {
    #[arguments(id: $id)]
    pub vault: Option<Vault>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Vault {
    pub id: cynic::Id,
    pub owner: Account,
    pub token_vaults: Option<Vec<TokenVault>>,
    pub deposits: Option<Vec<VaultDeposit>>,
    pub withdraws: Option<Vec<VaultWithdraw>>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct VaultWithdraw {
    pub id: cynic::Id,
    pub sender: Account,
    pub transaction: Transaction,
    pub timestamp: BigInt,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
    pub requested_amount: BigInt,
    pub requested_amount_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct VaultDeposit {
    pub id: cynic::Id,
    pub sender: Account,
    pub transaction: Transaction,
    pub timestamp: BigInt,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Transaction {
    pub id: cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct TokenVault {
    pub id: cynic::Id,
    pub balance: BigInt,
    pub balance_display: BigDecimal,
    pub token: Erc20,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Account {
    pub id: Bytes,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
