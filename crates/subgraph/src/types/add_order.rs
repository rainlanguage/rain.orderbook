use super::common::{SgAddOrderWithOrder, SgBytes};
use crate::schema;
use serde::Serialize;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(cynic::QueryVariables, Debug)]
pub struct TransactionAddOrdersVariables {
    pub id: SgBytes,
}

#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "Query", variables = "TransactionAddOrdersVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SgTransactionAddOrdersQuery {
    #[arguments(where: { transaction_: { id: $id } })]
    pub add_orders: Vec<SgAddOrderWithOrder>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(SgTransactionAddOrdersQuery);
