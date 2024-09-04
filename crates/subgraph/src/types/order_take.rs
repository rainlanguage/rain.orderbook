use super::common::*;
use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "PaginationWithIdQueryVariables")]
#[typeshare]
pub struct OrderTakesListQuery {
    #[arguments(skip: $skip, first: $first, orderBy: "timestamp", orderDirection: "desc", where: { order_: { id: $id } })]
    pub trades: Vec<Trade>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "IdQueryVariables")]
#[typeshare]
pub struct OrderTakeDetailQuery {
    #[arguments(id: $id)]
    pub trade: Option<Trade>,
}
