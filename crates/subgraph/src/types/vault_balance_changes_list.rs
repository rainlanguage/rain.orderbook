use crate::schema;
use serde::Serialize;
use typeshare::typeshare;
#[derive(cynic::QueryVariables, Debug, Clone)]
#[typeshare]
pub struct VaultBalanceChangesListQueryVariables {
    pub first: Option<i32>,
    pub id: Bytes,
    pub skip: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(
    graphql_type = "Query",
    variables = "VaultBalanceChangesListQueryVariables"
)]
#[typeshare]
pub struct VaultBalanceChangesListQuery {
    #[arguments(orderDirection: "desc", where: { vault_: { id: $id } }, skip: $skip, first: $first)]
    pub vault_balance_changes: Vec<VaultBalanceChange>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct VaultBalanceChange {
    pub __typename: String,
    pub amount: BigInt,
    pub new_vault_balance: BigInt,
    pub old_vault_balance: BigInt,
    pub vault: Vault,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Vault {
    pub id: Bytes,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
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
