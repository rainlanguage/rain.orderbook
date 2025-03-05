use super::common::*;
use crate::schema;
use serde::Serialize;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "Query", variables = "SgIdQueryVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgTransactionDetailQuery {
    #[arguments(id: $id)]
    pub transaction: Option<SgTransaction>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(SgTransactionDetailQuery);
