use crate::schema;
use typeshare::typeshare;
#[derive(cynic::QueryVariables, Debug)]
pub struct VaultBalanceChangesListQueryVariables<'a> {
    pub first: Option<i32>,
    pub id: &'a str,
    pub skip: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Query",
    variables = "VaultBalanceChangesListQueryVariables"
)]
pub struct VaultBalanceChangesListQuery {
    #[arguments(orderDirection: "desc", where: { vault_: { id: $id } }, skip: $skip, first: $first)]
    pub vault_balance_changes: Vec<VaultBalanceChange>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct VaultBalanceChange {
    pub __typename: String,
    pub amount: BigInt,
    pub new_vault_balance: BigInt,
    pub old_vault_balance: BigInt,
    pub vault: Vault,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Vault {
    pub id: Bytes,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum OrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
