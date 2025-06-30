use super::common::*;
use crate::schema;
use serde::Serialize;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::*;

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "SgVaultsListQueryVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgVaultsListQuery {
    #[arguments(orderBy: "id", orderDirection: "desc", skip: $skip, first: $first, where: $filters)]
    pub vaults: Vec<SgVault>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "SgIdQueryVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgVaultDetailQuery {
    #[arguments(id: $id)]
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub vault: Option<SgVault>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "SgPaginationWithIdQueryVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SgVaultBalanceChangesListQuery {
    #[arguments(orderDirection: "desc", orderBy: "timestamp", where: { vault_: { id: $id } }, skip: $skip, first: $first)]
    pub vault_balance_changes: Vec<SgVaultBalanceChangeUnwrapped>,
}
