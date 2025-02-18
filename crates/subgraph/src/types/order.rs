use super::common::*;
use crate::schema;
use serde::Serialize;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::*;

#[derive(cynic::QueryVariables, Debug)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgBatchOrderDetailQueryVariables {
    #[cynic(rename = "id_list")]
    pub id_list: SgOrderIdList,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "Order_filter")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgOrderIdList {
    #[cynic(rename = "id_in")]
    pub id_in: Vec<SgBytes>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "SgBatchOrderDetailQueryVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgBatchOrderDetailQuery {
    #[arguments(where: $id_list)]
    pub orders: Vec<SgOrder>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "SgOrdersListQueryVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgOrdersListQuery {
    #[arguments(orderBy: "timestampAdded", orderDirection: "desc", skip: $skip, first: $first, where: $filters)]
    pub orders: Vec<SgOrder>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "SgIdQueryVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgOrderDetailQuery {
    #[arguments(id: $id)]
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub order: Option<SgOrder>,
}
