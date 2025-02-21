use super::common::{SgBytes, SgRemoveOrderWithOrder};
use crate::schema;
use serde::Serialize;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(cynic::QueryVariables, Debug)]
pub struct TransactionRemoveOrdersVariables {
    pub id: SgBytes,
}

#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "Query", variables = "TransactionRemoveOrdersVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SgTransactionRemoveOrdersQuery {
    #[arguments(where: { transaction_: { id: $id } })]
    pub remove_orders: Vec<SgRemoveOrderWithOrder>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(SgTransactionRemoveOrdersQuery);
