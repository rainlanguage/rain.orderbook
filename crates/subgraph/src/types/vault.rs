use super::common::*;
use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "VaultsListQueryVariables")]
#[typeshare]
pub struct VaultsListQuery {
    #[arguments(orderBy: "id", orderDirection: "desc", skip: $skip, first: $first, where: $filters)]
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

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(
    graphql_type = "Query",
    variables = "PaginationWithTimestampQueryVariables"
)]
#[typeshare]
pub struct VaultBalanceChangesByTimeListQuery {
    #[arguments(
        skip: $skip,
        first: $first,
        orderDirection: "asc",
        orderBy: "timestamp",
        where: {
            vault_: { id: $id },
            timestamp_gte: $timestamp_gte,
            timestamp_lte: $timestamp_lte
        }
    )]
    pub vault_balance_changes: Vec<VaultBalanceChangeUnwrapped>,
}
