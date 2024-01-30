use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryVariables, Debug)]
pub struct VaultDetailQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "Query", variables = "VaultDetailQueryVariables")]
pub struct VaultDetailQuery {
    #[arguments(id: $id)]
    pub token_vault: Option<TokenVault>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(variables = "VaultDetailQueryVariables")]
pub struct TokenVault {
    pub id: cynic::Id,
    pub owner: Account,
    pub balance: BigInt,
    pub balance_display: BigDecimal,
    pub token: Erc20,
    pub vault: Vault,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(variables = "VaultDetailQueryVariables")]
pub struct Vault {
    pub id: cynic::Id,
    pub vault_id: BigInt,
    #[arguments(where: { tokenVault_: { id: $id } })]
    pub deposits: Option<Vec<VaultDeposit>>,
    #[arguments(where: { tokenVault_: { id: $id } })]
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
    pub token_vault: TokenVault2,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "TokenVault")]
pub struct TokenVault2 {
    pub balance_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct VaultDeposit {
    pub id: cynic::Id,
    pub transaction: Transaction,
    pub timestamp: BigInt,
    pub sender: Account,
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
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub id: cynic::Id,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Account {
    pub id: Bytes,
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
