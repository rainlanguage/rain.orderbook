use super::common::*;
use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(
    graphql_type = "Query",
    variables = "PaginationWithTimestampQueryVariables"
)]
#[typeshare]
pub struct OrderTradesListQuery {
    #[arguments(
        skip: $skip,
        first: $first,
        orderBy: "timestamp",
        orderDirection: "desc",
        where: {
            order_: { id: $id },
            timestamp_gte: $timestamp_gte,
            timestamp_lte: $timestamp_lte
        }
    )]
    pub trades: Vec<Trade>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "IdQueryVariables")]
#[typeshare]
pub struct OrderTradeDetailQuery {
    #[arguments(id: $id)]
    pub trade: Option<Trade>,
}
